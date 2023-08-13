use shipyard::*;
use crate::*;

fn modify_system(mut view_life: ViewMut<Life>) {
    for (id, mut life) in (&mut view_life).iter().with_id() {
        if life.0 < 0 {
            life.0 = 0;
        }
    }
}
fn modified_system(mut view_life: ViewMut<Life>) {
    let ids: Vec<_> =
        view_life.modified().iter().ids().collect();
    for id in ids {
        view_life.remove(id);
    }
}
fn modified_workload() -> Workload {
    (modify_system, modified_system).into_workload()
}

// fn insert_system(mut view_life: ViewMut<Life>, mut deads: ViewMut<Dead>) {
//     for (id, mut life) in (&mut view_life).iter().with_id() {
//         if life.0 < 0 {
//             deads.add_component_unchecked(id, Dead);
//         }
//     }
// }
// fn inserted_system(mut view_life: ViewMut<Life>, mut deads: ViewMut<Dead>) {
//     assert_eq!(deads.inserted().iter().count(), 2);
//     // let ids: Vec<_> =
//     //     deads.inserted().iter().ids().collect();
//     // for id in ids {
//     //     view_life.remove(id);
//     //     deads.remove(id);
//     // }
// }
// fn inserted_workload() -> Workload {
//     (insert_system, inserted_system).into_workload()
// }

mod tests {
    use shipyard::*;
    use crate::tracking_test::*;
    #[test]
    fn modified_test() {
        let mut world = World::new();
        world.add_entity(Life(3));
        world.add_entity(Life(2));
        world.add_entity(Life(-2));
        world.add_entity(Life(1));
        world.add_entity(Life(-1));
        world.add_workload(modified_workload);

        world.run_workload(modified_workload).unwrap();

        world.run(|view_life: View<Life>| {
            assert_eq!(view_life.iter().count(), 3);
        });
    }
    //
    // #[test]
    // fn inserted_test() {
    //     let mut world = World::new();
    //     world.add_entity((Life(3), Dead));
    //     world.add_entity((Life(2), Dead));
    //     world.add_entity((Life(-2), Dead));
    //     world.add_entity((Life(1), Dead));
    //     world.add_entity((Life(-1), Dead));
    //     world.add_workload(inserted_workload);
    //
    //     world.run_workload(inserted_workload).unwrap();
    //
    //     world.run(|view_life: View<Life>| {
    //         assert_eq!(view_life.iter().count(), 3);
    //     });
    // }
}