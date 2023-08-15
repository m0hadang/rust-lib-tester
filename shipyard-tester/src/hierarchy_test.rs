use super::Pos;
use shipyard::*;
use std::cmp::PartialOrd;

// 부모 컴포넌트
#[derive(Component)]
struct Parent {
    num_children: usize, // 자식 갯수
    first_child: EntityId,// 첫번째 자식 엔티티 ID
}

// 자식 컴포넌트
#[derive(Component)]
struct Child {
    parent: EntityId,
    prev: EntityId, // 형제 체인을 원형으로 만들어 옵션을 피함
    next: EntityId,
}

/*
!!! 엔티티 !!!
부모 엔티티, 자식 엔티티라는 용어를 쓰기도 한다.
하지만 이는 EOS가 아닌 객체 지향적인 개념이며 잘못된 표현이다.

엔티티는 오직 ID이다. 부모 컴포넌트는 자식 엔티티 ID를 값으로 가지고 있는 컴포넌트일 뿐이다.
자식 컴포넌트 역시 부모와 형제들 엔티티 ID를 가지고 있는 컴포넌트일 뿐이다.

부모 엔티티, 자식 엔티티라고 표현하는 것은 개발자가 해당 ID를 부모, 자식 개념으로 바라보는 것이다.
엔티티는 절대 클래스나 인스턴스가 아닌 단순한 ID이다.

만약 표현한다면 부모, 자식으로 표현 하는 것이 맞을 것이다.
부모를 만들기 위해 부모 컴포넌트가 필요하며 부모 엔티티 ID가 할당된다.
*/

trait Hierarchy {
    // Removes the child status of an entity.
    fn detach(&mut self, id: EntityId);
    // Attaches an entity as a child to a given parent entity.
    fn attach(&mut self, id: EntityId, parent: EntityId);
    fn attach_new(&mut self, parent: EntityId) -> EntityId;
    fn remove(&mut self, id: EntityId);
    fn remove_all(&mut self, id: EntityId);
    fn sort_children_by<F>(&mut self, id: EntityId, compare: F)
        where
            F: FnMut(&EntityId, &EntityId) -> std::cmp::Ordering;
}

impl Hierarchy for (EntitiesViewMut<'_>, ViewMut<'_, Parent>, ViewMut<'_, Child>) {
    fn detach(&mut self, id: EntityId) {
        let (
            _,
            parents,
            children
        ) = self;

        // 자식 컴포넌트 제거
        if let Some(child) = children.remove(id) {
            // 부모 컴포넌트에서 자식 갯수 1나 감소
            let parent = &mut parents[child.parent];
            parent.num_children -= 1;

            if parent.num_children == 0 {
                // 자식 없으면 부모 컴포넌트도 제거
                parents.remove(child.parent);
            } else {
                if parent.first_child == id {
                    // 삭제된 자식이 첫번쨰이면 다음 자식을 첫번째로 설정
                    parent.first_child = child.next;
                }
                // 삭제된 자식을 제외하는 링크 결함
                children[child.prev].next = child.next;
                children[child.next].prev = child.prev;
            }
        }
    }
    fn attach(&mut self, id: EntityId, parent: EntityId) {
        // 기존 링크 관계 제거
        self.detach(id);

        let (
            entities,
            parents,
            children
        ) = self;

        if let Ok(mut p) = parents.get(parent) {
            //부모에 자식 추가
            p.num_children += 1;

            let prev = children[p.first_child].prev;
            let next = p.first_child;

            children[prev].next = id;
            children[next].prev = id;

            entities.add_component(id, children, Child { parent, prev, next });
        } else {
            // 새로 부모 추가 + 부모에 자식 추가
            entities.add_component(
                id,
                children,
                Child {
                    parent,
                    prev: id,
                    next: id,
                },
            );
            entities.add_component(
                parent,
                parents,
                Parent {
                    num_children: 1,
                    first_child: id,
                },
            );
        }
    }
    fn attach_new(&mut self, parent: EntityId) -> EntityId {
        // 부모에 자식 생성하여 추가
        let id = self.0.add_entity((), ());
        self.attach(id, parent);
        id
    }
    fn remove(&mut self, id: EntityId) {
        // 자식 링크 제거
        self.detach(id);

        // 형제와 형제의 자식들
        let children = (&self.1, &self.2).children(id).collect::<Vec<_>>();
        for child_id in children {
            self.detach(child_id);
        }
        self.1.remove(id);
    }
    fn remove_all(&mut self, id: EntityId) {
        let (
            _,
            parents,
            children
        ) = self;

        for child_id in (&*parents, &*children).children(id).collect::<Vec<_>>() {
            self.remove_all(child_id);
        }
        self.remove(id);
    }
    fn sort_children_by<F>(&mut self, id: EntityId, compare: F)
        where
            F: FnMut(&EntityId, &EntityId) -> std::cmp::Ordering,
    {
        let (
            _,
            parents,
            children_storage
        ) = self;

        let mut children = (&*parents, &*children_storage)
            .children(id)
            .collect::<Vec<EntityId>>();
        if children.len() > 1 {
            children.sort_by(compare);
            // set first_child in Parent component
            parents[id].first_child = children[0];
            // loop through children and relink them
            for i in 0..children.len() - 1 {
                children_storage[children[i]].next = children[i + 1];
                children_storage[children[i + 1]].prev = children[i];
            }
            children_storage[children[0]].prev = *children.last().unwrap();
            children_storage[*children.last().unwrap()].next = children[0];
        }
    }
}

// 자식들 열거자
struct ChildrenIter<C> {
    get_child: C,// 자식 View로 설정
    cursor: (EntityId, usize),// (첫번째 자식 Entity ID, 자식 갯수)로 설정
}
impl<'a, C> Iterator for ChildrenIter<C>
    where
        C: Get<Out = &'a Child> + Copy,
{
    type Item = EntityId;// 항목은 EntityId

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.1 > 0 { // 자식 갯수
            self.cursor.1 -= 1;
            let ret = self.cursor.0;// 해당 Entity ID 반환
            self.cursor.0 = self.get_child.get(self.cursor.0).unwrap().next;//
            Some(ret)
        } else {
            None
        }
    }
}

struct AncestorIter<C> {
    get_child: C,// 자식 View로 설정
    cursor: EntityId,// 현재 Entity ID로 설정
}

impl<'a, C> Iterator for AncestorIter<C>
    where
        C: Get<Out = &'a Child> + Copy,
{
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_child.get(self.cursor).ok().map(|child| {
            self.cursor = child.parent;
            child.parent
        })
    }
}

struct DescendantsIter<P, C> {
    get_parent: P,// 부모 View로 설정
    get_child: C,// 자식 View로 설정
    cursors: Vec<(EntityId, usize)>,// [(parent.first_child, parent.num_children)]로 설정
}

impl<'a, P, C> Iterator for DescendantsIter<P, C>
    where
        P: Get<Out = &'a Parent> + Copy,
        C: Get<Out = &'a Child> + Copy,
{
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cursor) = self.cursors.last_mut() {
            if cursor.1 > 0 {
                cursor.1 -= 1;
                let ret = cursor.0;
                cursor.0 = self.get_child.get(cursor.0).unwrap().next;
                if let Ok(parent) = self.get_parent.get(ret) {
                    self.cursors.push((parent.first_child, parent.num_children));
                }
                Some(ret)
            } else {
                self.cursors.pop();
                self.next()
            }
        } else {
            None
        }
    }
}

trait HierarchyIter<'a, P, C> {
    fn ancestors(&self, id: EntityId) -> AncestorIter<C>;//조상들
    fn children(&self, id: EntityId) -> ChildrenIter<C>;//자식들
    fn descendants(&self, id: EntityId) -> DescendantsIter<P, C>;//자손들
}

// P : 부모
// C : 자식
// (P, C) : P, C 형태의 튜플에 대한 구현
impl<'a, P, C> HierarchyIter<'a, P, C> for (P, C)
    where
        P: Get<Out = &'a Parent> + Copy,
        C: Get<Out = &'a Child> + Copy,
{
    fn ancestors(&self, id: EntityId) -> AncestorIter<C> {
        let (_, children) = self;

        AncestorIter {
            get_child: *children,
            cursor: id,
        }
    }

    fn children(&self, id: EntityId) -> ChildrenIter<C> {
        let (parents, children) = self;

        ChildrenIter {
            get_child: *children,
            cursor: parents
                .get(id)
                .map_or((id, 0), |parent| (parent.first_child, parent.num_children)),
        }
    }

    fn descendants(&self, id: EntityId) -> DescendantsIter<P, C> {
        let (parents, children) = self;

        DescendantsIter {
            get_parent: *parents,
            get_child: *children,
            cursors: parents.get(id).map_or_else(
                |_| Vec::new(),
                |parent| vec![(parent.first_child, parent.num_children)],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::hierarchy_test::*;

    #[test]
    fn basic() {
        let world = World::new();

        let mut hierarchy = world
            .borrow::<(EntitiesViewMut, ViewMut<Parent>, ViewMut<Child>)>()
            .unwrap();

        let root1 = hierarchy.0.add_entity((), ());
        let root2 = hierarchy.0.add_entity((), ());

        let e1 = hierarchy.attach_new(root1);
        let _e2 = hierarchy.attach_new(e1);
        let e3 = hierarchy.attach_new(e1);
        let _e4 = hierarchy.attach_new(e3);

        hierarchy.attach(e3, root2);
    }
    #[test]
    fn test_hierarchy() {
        let world = World::new();

        let mut hierarchy = world
            .borrow::<(EntitiesViewMut, ViewMut<Parent>, ViewMut<Child>)>()
            .unwrap();

        let root1 = hierarchy.0.add_entity((), ());
        let root2 = hierarchy.0.add_entity((), ());

        let e1 = hierarchy.attach_new(root1);
        let e2 = hierarchy.attach_new(e1);
        let e3 = hierarchy.attach_new(e1);
        let e4 = hierarchy.attach_new(e3);

        hierarchy.attach(e3, root2);

        let e5 = hierarchy.attach_new(e3);

        assert!((&hierarchy.1, &hierarchy.2)
            .children(e3)
            .eq([e4, e5].iter().cloned()));

        assert!((&hierarchy.1, &hierarchy.2)
            .ancestors(e4)
            .eq([e3, root2].iter().cloned()));

        assert!((&hierarchy.1, &hierarchy.2)
            .descendants(root1)
            .eq([e1, e2].iter().cloned()));

        assert!((&hierarchy.1, &hierarchy.2)
            .descendants(root2)
            .eq([e3, e4, e5].iter().cloned()));

        hierarchy.detach(e1);

        assert!((&hierarchy.1, &hierarchy.2).descendants(root1).eq(None));
        assert!((&hierarchy.1, &hierarchy.2).ancestors(e1).eq(None));
        assert!((&hierarchy.1, &hierarchy.2)
            .children(e1)
            .eq([e2].iter().cloned()));

        hierarchy.remove(e1);

        assert!((&hierarchy.1, &hierarchy.2).children(e1).eq(None));

        hierarchy.remove_all(root2);

        assert!((&hierarchy.1, &hierarchy.2).descendants(root2).eq(None));
        assert!((&hierarchy.1, &hierarchy.2).descendants(e3).eq(None));
        assert!((&hierarchy.1, &hierarchy.2).ancestors(e5).eq(None));
    }

    #[test]
    fn test_sorting() {
        let world = World::new();

        let (mut hierarchy, mut vm_pos) = world
            .borrow::<(
                (EntitiesViewMut, ViewMut<Parent>, ViewMut<Child>),
                ViewMut<Pos>,
            )>()
            .unwrap();

        let root = hierarchy.0.add_entity((), ());

        let e0 = hierarchy.attach_new(root);
        let e1 = hierarchy.attach_new(root);
        let e2 = hierarchy.attach_new(root);
        let e3 = hierarchy.attach_new(root);
        let e4 = hierarchy.attach_new(root);

        hierarchy.0.add_component(e0, &mut vm_pos, Pos(7, 0));
        hierarchy.0.add_component(e1, &mut vm_pos, Pos(5, 0));
        hierarchy.0.add_component(e2, &mut vm_pos, Pos(6, 0));
        hierarchy.0.add_component(e3, &mut vm_pos, Pos(1, 0));
        hierarchy.0.add_component(e4, &mut vm_pos, Pos(3, 0));

        assert!((&hierarchy.1, &hierarchy.2)
            .children(root)
            .eq([e0, e1, e2, e3, e4].iter().cloned()));

        hierarchy.sort_children_by(root, |a, b| {
            vm_pos[*a].0.partial_cmp(&vm_pos[*b].0).unwrap()
        });

        assert!((&hierarchy.1, &hierarchy.2)
            .children(root)
            .eq([e3, e4, e1, e2, e0].iter().cloned()));
    }
}