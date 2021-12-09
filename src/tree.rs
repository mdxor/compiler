#[cfg(test)]
use serde::Serialize;
#[cfg_attr(test, derive(Serialize))]
pub struct Node<T> {
  pub child: Option<usize>,
  pub last_child: Option<usize>,
  pub prev: Option<usize>,
  pub next: Option<usize>,
  pub item: T,
}

#[cfg_attr(test, derive(Serialize))]
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
      last_child: None,
      prev: None,
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
      last_child: None,
      prev: None,
      next: None,
      item,
    });
    len
  }

  pub fn append(&mut self, item: T) -> usize {
    let next_index = self.create_node(item);
    let next = Some(next_index);
    if let Some(index) = self.cur {
      self[index].next = next;
      self[next_index].prev = self.cur;
      if let Some(&parent) = self.spine.last() {
        self[parent].last_child = next;
      }
    } else if let Some(&parent) = self.spine.last() {
      self[parent].child = next;
      self[parent].last_child = next;
    }
    self.cur = next;
    next_index
  }

  pub fn peek_up(&self) -> Option<usize> {
    self.spine.last().copied()
  }

  pub fn lower(&mut self) -> Option<usize> {
    let cur_index = self.cur.unwrap();
    self.spine.push(cur_index);
    self.cur = self[cur_index].child;
    self.cur
  }

  pub fn lower_to_last(&mut self) -> Option<usize> {
    let cur_index = self.cur.unwrap();
    self.spine.push(cur_index);
    self.cur = self[cur_index].last_child;
    self.cur
  }

  pub fn raise(&mut self) -> Option<usize> {
    let index = Some(self.spine.pop()?);
    self.cur = index;
    index
  }

  pub fn pop(&mut self) -> Option<usize> {
    let cur = self.cur.unwrap();
    let prev = self[cur].prev;
    if let Some(prev_index) = prev {
      self[prev_index].next = None;
    }
    self.nodes.pop();
    prev
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
