pub enum NextTraverseVisit {
  /// exit tree traverse
  Exit,
  /// continue children
  VisitChildren,
  /// continue siblings, skip children
  SkipChildren,
}

pub trait AbstractTree {
  fn visit_children(&self, visitor: impl FnMut(&Self));
  fn children_count(&self) -> usize {
    let mut visit_count = 0;
    self.visit_children(|_| visit_count += 1);
    visit_count
  }
  fn has_children(&self) -> bool {
    self.children_count() == 0
  }
  /// Check if this node is a leaf of the tree.
  fn is_leaf(&self) -> bool {
    !self.has_children()
  }

  fn traverse(&self, visitor: &mut impl FnMut(&Self)) {
    visitor(self);
    self.visit_children(|child| child.traverse(visitor))
  }

  /// leaf_visitor: when meet leaf, return if should continue visit tree
  fn traverse_by_branch_leaf(
    &self,
    mut branch_enter_visitor: impl FnMut(&Self) -> NextTraverseVisit,
    mut leaf_visitor: impl FnMut(&Self) -> bool,
  ) where
    Self: Clone,
  {
    let mut stack = Vec::new(); // todo estimate depth for allocation
    stack.push(self.clone());

    while let Some(node) = stack.pop() {
      match branch_enter_visitor(&node) {
        NextTraverseVisit::Exit => return,
        NextTraverseVisit::VisitChildren => {
          if node.is_leaf() {
            if !leaf_visitor(&node) {
              return;
            }
          } else {
            node.visit_children(|child| stack.push(child.clone()));
          }
        }
        NextTraverseVisit::SkipChildren => {}
      }
    }
  }

  fn traverse_pair_subtree(
    &self,
    visitor: &mut impl FnMut(&Self, Option<&Self>) -> NextTraverseVisit,
  ) where
    Self: AbstractParentTree + Clone,
  {
    use NextTraverseVisit::*;
    let mut stack = Vec::new();
    stack.push(self.clone());

    while let Some(node) = stack.pop() {
      let next = if let Some(parent) = node.get_parent() {
        visitor(&node, Some(&parent))
      } else {
        visitor(&node, None)
      };

      match next {
        Exit => return,
        VisitChildren => node.visit_children(|child| stack.push(child.clone())),
        SkipChildren => continue,
      };
    }
  }

  /// Get the tree depth starting with this node. (not including self)
  fn get_max_children_depth(&self) -> usize {
    if self.is_leaf() {
      return 0;
    }

    let mut max_depth = 0;
    self.visit_children(|child| max_depth = max_depth.max(child.get_max_children_depth()));
    max_depth + 1
  }
}
pub trait AbstractTreeMut {
  fn visit_children_mut(&mut self, visitor: impl FnMut(&mut Self));
  fn traverse_mut(&mut self, visitor: &mut impl FnMut(&mut Self)) {
    visitor(self);
    self.visit_children_mut(|child| child.traverse_mut(visitor))
  }
}

pub trait AbstractParentTree: Sized {
  /// this actually requires self is cheap to create/clone
  fn get_parent(&self) -> Option<Self>;

  fn get_tree_depth(&self) -> usize {
    let mut count = 0;
    self.traverse_parent(&mut |_| count += 1);
    count - 1
  }

  fn traverse_parent(&self, visitor: &mut impl FnMut(&Self)) {
    visitor(self);
    if let Some(parent) = self.get_parent() {
      parent.traverse_parent(visitor)
    }
  }

  fn traverse_pair_parent_chain(&self, visitor: &mut impl FnMut(&Self, Option<&Self>)) {
    if let Some(parent) = self.get_parent() {
      visitor(self, Some(&parent));
      parent.traverse_pair_parent_chain(visitor)
    } else {
      visitor(self, None);
    }
  }
}
