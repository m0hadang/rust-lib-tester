mod entity_test;
mod component_test;
mod control_component_test;
mod tracking_test;
mod workload_test;

use shipyard::*;

#[derive(Component, Debug)]
struct Pos(u32, u32);
impl Pos {
    fn new(x: u32, y: u32) -> Pos {
        Pos(x, y)
    }
}

#[derive(Component, Debug)]
struct Vel(u32);
impl Vel {
    fn new(velocity: u32) -> Vel {
        Vel(velocity)
    }
}

// unique component
// Unique components (a.k.a. resources) are useful
// when you know there will only ever be a single instance of some component.
// In that case there is no need to attach the component to an entity.
// It also works well as global data without most of its drawback.
#[derive(Unique)]
struct Camera(String);
impl Camera {
    fn new(name: &str) -> Camera {
        Camera(name.to_string())
    }
}

#[derive(Component)]
#[track(Modification)]
struct Life(i32);
impl Life {
    fn new(life: i32) -> Life {
        Life(life)
    }
}

#[derive(Component)]
#[track(Insertion)]
struct Dead;

fn read_only_system_1(
    view_vel: View<Vel>) {
    view_vel.iter().for_each(|vel|{
        println!("O : vel : {}", vel.0);
    });
}
fn read_only_system_2(
    view_vel: View<Vel>) {
    view_vel.iter().for_each(|vel|{
        println!("X : vel : {}", vel.0);
    });
}

fn read_write_system(mut view_vel: ViewMut<Vel>) {
    for mut vel in (&mut view_vel).iter() {
        vel.0 = 10;
        println!("X : vel : {}", vel.0);
    }
}

fn parallel_workload() -> Workload {
    // outer-parallelism
    //
    // Workloads will run their systems
    // first to last and try to run them in parallel when possible.
    (read_only_system_1, read_only_system_2).into_workload()
}
fn not_parallel_workload() -> Workload {
    (read_only_system_1, read_write_system).into_workload()
}

pub fn outer_parallel_able_test() {
    let mut world = World::new();
    world.add_entity((Vel::new(0), Pos::new(0, 0)));
    world.add_entity((Vel::new(1), Pos::new(1, 1)));
    world.add_entity((Vel::new(2), Pos::new(2, 2)));
    world.add_workload(parallel_workload);
    world.run_workload(parallel_workload).unwrap();
    // X : vel : 0
    // O : vel : 0
    // O : vel : 1
    // O : vel : 2
    // X : vel : 1
    // X : vel : 2
}

pub fn outer_parallel_not_able_test() {
    // - Systems accessing AllStorages stop all threading.
    // - There can't be any other access during an exclusive access,
    // so ViewMut<T> will block T threading.
    let mut world = World::new();
    world.add_entity((Vel::new(0), Pos::new(0, 0)));
    world.add_entity((Vel::new(1), Pos::new(1, 1)));
    world.add_entity((Vel::new(2), Pos::new(2, 2)));
    world.add_workload(not_parallel_workload);
    world.run_workload(not_parallel_workload).unwrap();
    // O : vel : 0
    // O : vel : 1
    // O : vel : 2
    // X : vel : 0
    // X : vel : 1
    // X : vel : 2

    // When you make a workload, all systems in it will be checked and batches
    // (groups of systems that don't conflict) will be created.
    //
    // add_to_world returns information about these batches and
    // why each system didn't get into the previous batch.
}