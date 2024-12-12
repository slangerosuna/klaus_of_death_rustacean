use crate::core::*;
use futures::future::poll_fn;
use std::cell::SyncUnsafeCell;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio::time::Instant;

pub struct Scheduler {
    init_systems: Vec<System>,
    update_systems: Vec<System>,
    fixed_update_systems: Vec<System>,
    close_systems: Vec<System>,

    init_execution_order: Vec<Vec<usize>>,
    update_execution_order: Vec<Vec<usize>>,
    fixed_update_execution_order: Vec<Vec<usize>>,
    close_execution_order: Vec<Vec<usize>>,

    execution_lock: SchedulerLock,

    fixed_update_interval: Duration,
    start_time: Instant,
    prev_time: SyncUnsafeCell<f64>,
}

struct SchedulerLock(Mutex<bool>);

impl SchedulerLock {
    async fn lock(&self) {
        loop {
            let guard = self.0.lock().await;

            if !*guard {
                break;
            }

            tokio::time::sleep(Duration::from_micros(100)).await;
        }
        let mut guard = self.0.lock().await;
        *guard = true;
    }

    async fn unlock(&self) {
        let mut guard = self.0.lock().await;
        *guard = false;
    }
}

impl Scheduler {
    pub async fn loop_fixed_update(&self, game_state: *mut GameState) {
        let mut time = Instant::now();

        loop {
            self.fixed_update(game_state).await;

            let dur = Instant::now().duration_since(time);

            if dur < self.fixed_update_interval {
                tokio::time::sleep(dur).await;
            } else {
                eprintln!(
                    "Fixed update overran by {:?}",
                    dur - self.fixed_update_interval
                );
            }

            time = Instant::now();
        }
    }

    pub fn new(fixed_update_interval: f64) -> Scheduler {
        Scheduler {
            init_systems: Vec::new(),
            update_systems: Vec::new(),
            fixed_update_systems: Vec::new(),
            close_systems: Vec::new(),

            init_execution_order: Vec::new(),
            update_execution_order: Vec::new(),
            fixed_update_execution_order: Vec::new(),
            close_execution_order: Vec::new(),

            execution_lock: SchedulerLock(Mutex::new(false)),

            fixed_update_interval: Duration::from_secs_f64(fixed_update_interval),
            start_time: Instant::now(),
            prev_time: SyncUnsafeCell::new(0.0),
        }
    }

    pub fn add_system(&mut self, system: System, system_type: SystemType) {
        match system_type {
            SystemType::Init => {
                self.init_systems.push(system);
                self.init_execution_order =
                    self.generate_execution_order_for_systems(&self.init_systems);
            }
            SystemType::Update => {
                self.update_systems.push(system);
                self.update_execution_order =
                    self.generate_execution_order_for_systems(&self.update_systems);
            }
            SystemType::FixedUpdate => {
                self.fixed_update_systems.push(system);
                self.fixed_update_execution_order =
                    self.generate_execution_order_for_systems(&self.fixed_update_systems);
            }
            SystemType::Close => {
                self.close_systems.push(system);
                self.close_execution_order =
                    self.generate_execution_order_for_systems(&self.close_systems);
            }
        };
    }

    // you need to ensure that you call `generate_execution_order` for the system to be run
    pub fn add_system_without_execution_order_generation(
        &mut self,
        system: System,
        system_type: SystemType,
    ) {
        match system_type {
            SystemType::Init => self.init_systems.push(system),
            SystemType::Update => self.update_systems.push(system),
            SystemType::FixedUpdate => self.fixed_update_systems.push(system),
            SystemType::Close => self.close_systems.push(system),
        };
    }

    pub async fn init(&mut self, game_state: &mut GameState) {
        let time = self.get_time();
        let dt = 0.0;
        unsafe {
            self.prev_time.get().write(time);
        }

        self.execution_lock.lock().await;
        for group in self.init_execution_order.iter() {
            Self::await_group(group, &self.init_systems, game_state, time, dt).await;
        }
        self.execution_lock.unlock().await;
    }

    pub async fn update(&self, game_state: &mut GameState) {
        let time = self.get_time();
        // this is ok because update and init are never run at the same time
        let dt = time - unsafe { *self.prev_time.get() };
        unsafe {
            self.prev_time.get().write(time);
        }

        // used to ensure that update and fixed_update don't run at the same time
        self.execution_lock.lock().await;
        for group in self.update_execution_order.iter() {
            Self::await_group(group, &self.update_systems, game_state, time, dt).await;
        }
        self.execution_lock.unlock().await;
    }

    pub async fn fixed_update(&self, game_state: *mut GameState) {
        let time = self.get_time();
        let dt = self.fixed_update_interval.as_secs_f64();

        // used to ensure that update and fixed_update don't run at the same time
        self.execution_lock.lock().await;
        for group in self.fixed_update_execution_order.iter() {
            Self::await_group(group, &self.fixed_update_systems, game_state, time, dt).await;
        }
        self.execution_lock.unlock().await;
    }

    pub async fn close(&self, game_state: &mut GameState) {
        let time = self.get_time();
        let dt = time - unsafe { *self.prev_time.get() };
        unsafe {
            self.prev_time.get().write(time);
        }

        self.execution_lock.lock().await;
        for group in self.close_execution_order.iter() {
            Self::await_group(group, &self.close_systems, game_state, time, dt).await;
        }
        self.execution_lock.unlock().await;
    }

    pub async unsafe fn force_unlock(&self) {
        self.execution_lock.unlock().await;
    }

    pub async unsafe fn force_lock(&self) {
        self.execution_lock.lock().await;
    }

    async fn await_group(
        group: &Vec<usize>,
        systems: &Vec<System>,
        game_state: *mut GameState,
        time: f64,
        dt: f64,
    ) {
        let mut futures = Vec::with_capacity(group.len());

        // Run all systems in the group
        for system_index in group.iter() {
            let system = &systems[*system_index];
            futures.push((system.system)(game_state, time, dt));
        }

        // Wait for all futures to complete
        poll_fn(|cx| {
            for future in futures.iter_mut() {
                if let std::task::Poll::Pending = future.as_mut().poll(cx) {
                    return std::task::Poll::Pending;
                }
            }
            std::task::Poll::Ready(())
        })
        .await;
    }

    pub fn generate_execution_order(&mut self) {
        self.init_execution_order = self.generate_execution_order_for_systems(&self.init_systems);
        self.update_execution_order =
            self.generate_execution_order_for_systems(&self.update_systems);
        self.fixed_update_execution_order =
            self.generate_execution_order_for_systems(&self.fixed_update_systems);
        self.close_execution_order = self.generate_execution_order_for_systems(&self.close_systems);
    }

    fn generate_execution_order_for_systems(&self, systems: &Vec<System>) -> Vec<Vec<usize>> {
        let mut execution_order = Vec::new();
        let mut visited = vec![false; systems.len()];

        for i in 0..systems.len() {
            if !visited[i] {
                let mut group = vec![i];
                let mut dissallowed_components = systems[i].args.clone();

                if !dissallowed_components.contains(&GameState::get_component_type()) {
                    for j in 0..systems.len() {
                        if !visited[j] {
                            let mut can_run = true;

                            for component in &systems[j].args {
                                if dissallowed_components.contains(component)
                                    || *component == GameState::get_component_type()
                                {
                                    can_run = false;
                                    break;
                                }
                            }
                            if can_run {
                                group.push(j);
                                visited[j] = true;
                                for component in &systems[j].args {
                                    if !dissallowed_components.contains(component) {
                                        dissallowed_components.push(*component);
                                    }
                                }
                            }
                        }
                    }
                }

                execution_order.push(group);
            }
        }

        execution_order
    }

    pub fn get_time(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}
