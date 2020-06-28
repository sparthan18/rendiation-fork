pub use arena::*;
use std::collections::BTreeSet;

pub struct ArenaGraph<T> {
  nodes_data: Arena<T>,
  nodes: Arena<ArenaGraphNode<T>>,
}

pub type ArenaGraphNodeHandle<T> = Handle<ArenaGraphNode<T>>;

pub struct ArenaGraphNode<T> {
  data_handle: Handle<T>,
  handle: ArenaGraphNodeHandle<T>,
  from: BTreeSet<ArenaGraphNodeHandle<T>>,
  to: BTreeSet<ArenaGraphNodeHandle<T>>,
}

impl<T> ArenaGraphNode<T> {
  pub fn handle(&self) -> ArenaGraphNodeHandle<T> {
    self.handle
  }

  pub fn data_handle(&self) -> Handle<T> {
    self.data_handle
  }

  pub fn from(&self) -> &BTreeSet<ArenaGraphNodeHandle<T>> {
    &self.from
  }

  pub fn to(&self) -> &BTreeSet<ArenaGraphNodeHandle<T>> {
    &self.to
  }

  pub fn new(data_handle: Handle<T>, handle: ArenaGraphNodeHandle<T>) -> Self {
    Self {
      data_handle,
      handle,
      from: BTreeSet::new(),
      to: BTreeSet::new(),
    }
  }
}

impl<T> ArenaGraph<T> {
  pub fn new() -> Self {
    Self {
      nodes_data: Arena::new(),
      nodes: Arena::new(),
    }
  }

  pub fn new_node(&mut self, node_data: T) -> ArenaGraphNodeHandle<T> {
    let handle = self.nodes_data.insert(node_data);
    let node = ArenaGraphNode::new(handle, Handle::from_raw_parts(0, 0));
    let handle = self.nodes.insert(node);
    self.nodes.get_mut(handle).unwrap().handle = handle; // improvements need
    handle
  }

  pub fn get_node(&self, handle: ArenaGraphNodeHandle<T>) -> &ArenaGraphNode<T> {
    self.nodes.get(handle).unwrap()
  }

  pub fn get_node_data_mut(&mut self, handle: Handle<T>) -> &mut T {
    self.nodes_data.get_mut(handle).unwrap()
  }

  pub fn get_node_data(&self, handle: Handle<T>) -> &T {
    self.nodes_data.get(handle).unwrap()
  }

  pub fn get_node_data_by_node(&self, handle: ArenaGraphNodeHandle<T>) -> &T {
    self.nodes_data.get(self.get_node(handle).data_handle).unwrap()
  }

  pub fn get_node_data_mut_by_node(&mut self, handle: ArenaGraphNodeHandle<T>) -> &mut T {
    self.nodes_data.get_mut(self.get_node(handle).data_handle).unwrap()
  }

  pub fn connect_node(&mut self, from: ArenaGraphNodeHandle<T>, to: ArenaGraphNodeHandle<T>) {
    let from_node = self.nodes.get_mut(from).unwrap();
    from_node.to.insert(to);

    let to_node = self.nodes.get_mut(to).unwrap();
    to_node.from.insert(from);
  }

  pub fn traverse_dfs_in_topological_order(
    &self,
    node: ArenaGraphNodeHandle<T>,
    visitor: &mut impl FnMut(&ArenaGraphNode<T>),
  ) {
    let mut unresolved: BTreeSet<ArenaGraphNodeHandle<T>> = BTreeSet::new();
    let mut visited: BTreeSet<ArenaGraphNodeHandle<T>> = BTreeSet::new();

    fn visit<T>(
      n_handle: ArenaGraphNodeHandle<T>,
      visited: &mut BTreeSet<ArenaGraphNodeHandle<T>>,
      unresolved: &mut BTreeSet<ArenaGraphNodeHandle<T>>,
      graph: &ArenaGraph<T>,
      visitor: &mut impl FnMut(&ArenaGraphNode<T>),
    ) {
      if visited.contains(&n_handle) {
        return;
      }
      if unresolved.contains(&n_handle) {
        panic!("graph contains loops"); // todo
      }

      unresolved.insert(n_handle);

      let node = graph.get_node(n_handle);
      node
        .from
        .iter()
        .for_each(|from_id| visit(*from_id, visited, unresolved, graph, visitor));

      unresolved.remove(&n_handle);
      visited.insert(n_handle);
      visitor(node)
    }

    visit(node, &mut visited, &mut unresolved, self, visitor);
  }

  pub fn topological_order_list(
    &self,
    node: ArenaGraphNodeHandle<T>,
  ) -> Vec<ArenaGraphNodeHandle<T>> {
    let mut list = Vec::new();
    self.traverse_dfs_in_topological_order(node, &mut |node| list.push(node.handle()));
    list
  }
}