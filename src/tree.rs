#[cfg(test)]
use serde::Serialize;
#[cfg_attr(test, derive(Serialize))]
pub struct Node<T> {
  pub id: usize,
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
      id: 0,
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
      id: len,
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

  pub fn peek_parent(&self) -> Option<usize> {
    self.spine.last().copied()
  }

  pub fn peek_grandparent(&self) -> Option<usize> {
    if self.spine.len() >= 2 {
      Some(self.spine[self.spine.len() - 2])
    } else {
      None
    }
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

  pub fn to_root_last_child(&mut self) {
    self.spine = vec![0];
    self.cur = self[0].last_child;
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn spine(&self) -> &Vec<usize> {
    &self.spine
  }

  pub fn next_sibling(&mut self) -> Option<usize> {
    self.cur = self[self.cur.unwrap()].next;
    self.cur
  }

  pub fn visit<F>(&mut self, mut callback: F)
  where
    F: FnMut(T),
  {
    self.spine = vec![];
    self.cur = Some(0);
    loop {
      if let Some(cur) = self.cur {
        callback(self[cur].item);
        if self[cur].child.is_some() {
          self.lower();
        } else {
          while self[cur].next.is_none() && self.spine.len() > 0 {
            self.raise();
          }
          self.next_sibling();
        }
      } else {
        return;
      }
    }
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
