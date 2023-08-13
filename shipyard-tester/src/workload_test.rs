use shipyard::*;
use crate::*;

fn decrease_vel_system(mut view_vel: ViewMut<Vel>) {
    for vel in (&mut view_vel).iter() {
        vel.0 -= 1;
    }
}

fn flag_deleted_vel_system(v_vel: View<Vel>, mut deads: ViewMut<Dead>) {
    for (id, i) in v_vel.iter().with_id() {
        if i.0 == 0 {
            deads.add_component_unchecked(id, Dead);
        }
    }
}

fn clear_deleted_vel_system(mut all_storages: AllStoragesViewMut) {
    all_storages.delete_any::<SparseSet<Dead>>();
}

fn filter_vel_workload() -> Workload {
    (flag_deleted_vel_system, clear_deleted_vel_system).into_workload()
}

fn main_workload() -> Workload {
    (decrease_vel_system, filter_vel_workload).into_workload()
}

mod tests {
    use shipyard::*;
    use crate::*;
    use crate::workload_test::*;

    #[test]
    fn get_and_modify_component_test() {
        let mut world = World::new();
        world.add_entity(Vel(3));
        world.add_entity(Vel(2));
        world.add_entity(Vel(1));
        world.add_workload(main_workload);

        world.run_workload(main_workload).unwrap();
        world.run(|entities: EntitiesViewMut| {
            assert_eq!(entities.iter().count(), 2);
        });

        world.run_workload(main_workload).unwrap();
        world.run(|entities: EntitiesViewMut| {
            assert_eq!(entities.iter().count(), 1);
        });

        world.run_workload(main_workload).unwrap();
        world.run(|entities: EntitiesViewMut| {
            assert_eq!(entities.iter().count(), 0);
        });
    }
}