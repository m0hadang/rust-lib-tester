#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn get_and_modify_component_test() {
        let mut world = World::new();
        let entity_id =
            world.add_entity((Vel::new(0), Pos::new(0, 0)));

        world.run(
            |mut view_pos: ViewMut<Pos>, mut view_vel: ViewMut<Vel>| {
                (&mut view_vel).get(entity_id).unwrap().0 += 1;

                let (pos, vel) = (&mut view_pos, &mut view_vel).get(entity_id).unwrap();
                pos.0 += 1;
                pos.1 += 1;
                vel.0 += 1;

                view_pos[entity_id].0 += 1;
            },
        );

        world.run(
            |view_pos: View<Pos>, view_vel: View<Vel>| {
                assert_eq!(view_pos[entity_id].0, 2);
                assert_eq!(view_pos[entity_id].1, 1);
                assert_eq!(view_vel[entity_id].0, 2);
            },
        );
    }

    #[test]
    fn iterator_test() {
        let mut world = World::new();
        world.add_entity((Vel::new(0), Pos::new(0, 0)));
        world.add_entity((Vel::new(1), Pos::new(1, 1)));
        world.add_entity((Vel::new(2), Pos::new(2, 2)));
        world.run(
            |view_pos: View<Pos>, view_vel: View<Vel>| {
                let mut i = 0;
                for vel in view_vel.iter() {
                    assert_eq!(vel.0, i);
                    i += 1;
                }

                let mut i = 0;
                for (pos, vel) in (&view_pos, &view_vel).iter() {
                    assert_eq!(pos.0, i);
                    assert_eq!(pos.1, i);
                    assert_eq!(vel.0, i);
                    i += 1;
                }
                // - with id iterator
                // iter().with_id()

                // - parallel iterator
                // par_iter().for_each(|i| {
                // });
            },
        );
    }
}