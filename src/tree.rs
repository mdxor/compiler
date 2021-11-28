#[derive(Debug, Clone, Copy)]
pub struct Node<T> {
  pub child: Option<usize>,
  pub next: Option<usize>,
  pub item: T,
}

#[derive(Clone)]
pub struct Tree<T> {
  nodes: Vec<Node<T>>,
  spine: Vec<usize>,
  cur: Option<usize>,
}

impl<T: Default> Tree<T> {
  pub fn new() -> Self {
    let mut nodes = vec![];
    nodes.push(Node {
      child: None,
      next: None,
      item: <T>::default(),
    });
    Tree {
      nodes,
      spine: vec![],
      cur: Some(0),
    }
  }

  pub fn cur(&mut self) -> Option<usize> {
    self.cur
  }

  pub fn create_node(&mut self, item: T) -> usize {
    let len = self.nodes.len();
    self.nodes.push(Node {
      child: None,
      next: None,
      item,
    });
    len
  }

  pub fn append(&mut self, item: T) -> usize {
    let next_index = Some(self.create_node(item));
    if let Some(index) = self.cur {
      self[index].next = next_index;
    } else if let Some(&parent) = self.spine.last() {
      self[parent].child = next_index;
    }
    self.cur = next_index;
    next_index.unwrap()
  }

  pub fn lower(&mut self) -> usize {
    let cur_index = self.cur.unwrap();
    self.spine.push(cur_index);
    self.cur = self[cur_index].child;
    self.cur.unwrap()
  }

  pub fn raise(&mut self) -> Option<usize> {
    let index = Some(self.spine.pop()?);
    self.cur = index;
    index
  }

  pub fn next_sibling(&mut self) -> Option<usize> {
    self.cur = self[self.cur.unwrap()].next;
    self.cur
  }
}

impl<T> std::ops::Index<usize> for Tree<T> {
  type Output = Node<T>;

  fn index(&self, index: usize) -> &Self::Output {
    self.nodes.index(index)
  }
}

impl<T> std::ops::IndexMut<usize> for Tree<T> {
  fn index_mut(&mut self, index: usize) -> &mut Node<T> {
    self.nodes.index_mut(index)
  }
}
