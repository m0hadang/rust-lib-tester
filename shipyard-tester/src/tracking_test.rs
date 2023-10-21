use shipyard::*;
use crate::*;

fn modify_system(mut view_life: ViewMut<Life>, mut view_composit: ViewMut<ComplexLife>) {
    for (id, mut life) in (&mut view_life).iter().with_id() {
        if life.0 < 0 {
            life.0 = 0;
        }
    }

    for (id, mut composit) in (&mut view_composit).iter().with_id() {
        let data = composit.data.get("item").unwrap();
        if *data < 0 {
            *composit.data.get_mut("item").unwrap() = 0;
        }
    }
}

fn modified_system(mut view_life: ViewMut<Life>, mut view_composit: ViewMut<ComplexLife>) {
    let ids1: Vec<_> =
        view_life.modified().iter().ids().collect();
    assert_eq!(ids1.iter().count(), 2);
    for id in ids1 {
        let life = view_life.remove(id).unwrap();
        assert_eq!(life.0, 0);
    }

    let ids2: Vec<_> =
        view_composit.modified().iter().ids().collect();
    assert_eq!(ids2.iter().count(), 2);
    for id in ids2 {
        let life = view_composit.remove(id).unwrap();
        let data = life.data.get("item").unwrap();
        assert_eq!(*data, 0);
    }
}

fn modified_workload() -> Workload {
    (
        modify_system
            .before_all(modified_system),
        modified_system,  // 의존성 순서를 정의. 순서에 문제가 발생하면 패닉 발생. 컴파일 타임에 의존성 문제 확인 가능
    ).into_workload()
}

fn remove_system(mut view_life: View<Life>, mut view_dead: ViewMut<Dead>) {
    for (id, mut life) in (&mut view_life).iter().with_id() {
        if life.0 > 0 {
            // deads.add_component_unchecked(id, Dead);
            view_dead.remove(id);
        }
    }
}

fn removed_system(mut view_life: View<Life>, mut view_dead: View<Dead>) {
    assert_eq!(view_dead.removed().count(), 3);
    for id in view_dead.removed() {
        if let Ok(x) = view_dead.get(id) {
            assert!(false);
        }
        let life = view_life.get(id).unwrap();
        if life.0 <= 0 {
            assert!(false);
        }
    }
}

fn remove_workload() -> Workload {
    (
        remove_system
            .before_all(removed_system),// !! removed_system을 추가한 것이 아닌 의존성 설정한 것이다.
        removed_system, // 이것이 추가다. 만약 시스템을 추가하지 않으면 패닉 발생.
    ).into_workload()
}

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

        world.add_entity(ComplexLife::new(3));
        world.add_entity(ComplexLife::new(2));
        world.add_entity(ComplexLife::new(-2));
        world.add_entity(ComplexLife::new(1));
        world.add_entity(ComplexLife::new(-1));

        world.add_workload(modified_workload);

        world.run_workload(modified_workload).unwrap();

        world.run(|view_life: ViewMut<Life>| {
            assert_eq!(view_life.iter().count(), 3);
        });
    }

    #[test]
    fn removed_test() {
        let mut world = World::new();
        world.add_entity((Life(3), Dead));
        world.add_entity((Life(2), Dead));
        world.add_entity((Life(-2), Dead));
        world.add_entity((Life(1), Dead));
        world.add_entity((Life(0), Dead));
        world.add_workload(remove_workload);
        world.run_workload(remove_workload).unwrap();
        world.run(|view_dead: View<Dead>| {
            assert_eq!(view_dead.iter().count(), 2);
        });
    }
}