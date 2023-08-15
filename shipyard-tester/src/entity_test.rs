#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn add_entity_test() {
        let mut world = World::new();
        let _empty_entity = world.add_entity(());
        let _single_component = world.add_entity(Pos::new(3, 3));
        let _multiple_components = world.add_entity((Pos::new(5, 5), Vel::new(10)));
        world.run(
            |mut entities: EntitiesViewMut, mut view_pos: ViewMut<Pos>, mut view_vel: ViewMut<Vel>| {
                let _empty_entity = entities.add_entity(
                    (),
                    ());
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
            |entities: EntitiesView| {
                assert_eq!(entities.iter().count(), 6);
            },
        );
    }
    #[test]
    fn iter_entity_with_borrow_test() {
        let mut world = World::new();
        let _empty_entity = world.add_entity(());
        let _single_component = world.add_entity(Pos::new(3, 3));
        let _multiple_components = world.add_entity((Pos::new(5, 5), Vel::new(10)));
        let (firsts, seconds) = world
            .borrow::<(View<Pos>, View<Vel>)>()
            .unwrap();

        assert_eq!((&firsts, &seconds).iter().count(), 1);
    }
    #[test]
    fn add_entity_with_borrow_test() {
        let mut world = World::new();
        let (mut entities, mut view_pos, mut view_vel) = world
            .borrow::<(EntitiesViewMut, ViewMut<Pos>, ViewMut<Vel>)>()
            .unwrap();

        entities.add_entity(
            (),
            ());
        entities.add_entity(
            &mut view_pos,
            Pos::new(3, 3));
        entities.add_entity(
            (&mut view_pos, &mut view_vel),
            (Pos::new(5, 5), Vel::new(10)));

        assert_eq!((&mut view_pos, &mut view_vel).iter().count(), 1);
    }
    #[test]
    fn delete_entity_test() {
        let mut world = World::new();
        let entity_id = world.add_entity(());
        world.delete_entity(entity_id);
        world.run(|mut all_storages: AllStoragesViewMut| {
            let entity_id = all_storages.add_entity(Pos::new(1, 2));
            all_storages.delete_entity(entity_id);
        });
        world.run(
            |entities: EntitiesView| {
                assert_eq!(entities.iter().count(), 0);
            },
        );
    }
}