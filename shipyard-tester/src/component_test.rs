#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn add_component_test() {
        let mut world = World::new();
        world.add_entity(Pos::new(3, 3));
        world.add_entity((Pos::new(5, 5), Vel::new(10)));
        world.run(
            |mut entities: EntitiesViewMut, mut view_pos: ViewMut<Pos>, mut view_vel: ViewMut<Vel>| {
                let _single_component =
                    entities.add_entity(
                        &mut view_pos,
                        Pos::new(3, 3));
                let _multiple_components =
                    entities.add_entity(
                        (&mut view_pos, &mut view_vel),
                        (Pos::new(5, 5), Vel::new(10)));
            },
        );
        world.run(
            |view_pos: View<Pos>, view_vel: View<Vel>| {
                assert_eq!(view_pos.iter().count(), 4);
                assert_eq!((&view_pos, &view_vel).iter().count(), 2);
            },
        );
    }

    #[test]
    fn remove_component_test() {
        let mut world = World::new();
        let entity_id =
            world.add_entity((Pos::new(5, 5), Vel::new(10)));

        let vel_component: (Option<Vel>,) =
            world.remove::<Vel>(entity_id);
        let pos_vel_component: (Option<Pos>, Option<Vel>) =
            world.remove::<(Pos, Vel)>(entity_id);
        assert!(pos_vel_component.1.is_none());
        assert_eq!(pos_vel_component.0.as_ref().unwrap().0, 5);
        assert_eq!(pos_vel_component.0.as_ref().unwrap().0, 5);
        assert_eq!(vel_component.0.unwrap().0, 10);

        world.run(
            |mut entities: EntitiesViewMut, mut view_pos: ViewMut<Pos>, mut view_vel: ViewMut<Vel>| {
                let entity_id =
                    entities.add_entity(
                        (&mut view_pos, &mut view_vel),
                        (Pos::new(5, 5), Vel::new(10)));
                let _vel_component: Option<Pos> =
                    view_pos.remove(entity_id);// not error
                let _pos_vel_component: (Option<Pos>, Option<Vel>) =
                    (&mut view_pos, &mut view_vel).remove(entity_id);
            },
        );
        world.run(
            |view_pos: View<Pos>, view_vel: View<Vel>| {
                assert_eq!(view_pos.iter().count(), 0);
                assert_eq!((&view_pos, &view_vel).iter().count(), 0);
            },
        );
    }

    #[test]
    fn delete_component_test() {
        let mut world = World::new();
        let entity_id =
            world.add_entity((Vel::new(10), Pos::new(5, 5)));
        let not_ret_1 = world.delete_component::<Vel>(entity_id);
        let not_ret_2 = world.delete_component::<(Pos, Vel)>(entity_id);
        assert_eq!(not_ret_1, ());
        assert_eq!(not_ret_2, ());

        world.run(
            |mut entities: EntitiesViewMut, mut view_pos: ViewMut<Pos>, mut view_vel: ViewMut<Vel>| {
                let entity_id =
                    entities.add_entity(
                        (&mut view_pos, &mut view_vel),
                        (Pos::new(5, 5), Vel::new(10)));
                view_pos.delete(entity_id);// not error
                (&mut view_pos, &mut view_vel).delete(entity_id);
            },
        );

        let entity_id =
            world.add_entity((Vel::new(10), Pos::new(5, 5)));
        world.strip(entity_id);
    }

    #[test]
    fn strip_component_test() {
        let mut world = World::new();
        let entity_id =
            world.add_entity((Vel::new(10), Pos::new(5, 5)));

        world.strip(entity_id);

        world.run(|mut all_storages: AllStoragesViewMut| {
            let id = all_storages.add_entity(
                (Vel::new(10), Pos::new(5, 5))
            );
            all_storages.strip(id);
        });

        world.run(
            |view_pos: View<Pos>, view_vel: View<Vel>| {
                assert_eq!((&view_pos, &view_vel).iter().count(), 0);
            },
        );
    }

    #[test]
    fn unique_component_test() {
        let world = World::new();
        world.add_unique(Camera::new("test camera"));
        world.run(|camera: UniqueView<Camera>| {
            assert_eq!(camera.0, "test camera");
        });
    }
}