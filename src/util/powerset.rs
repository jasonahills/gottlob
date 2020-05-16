use std::collections::HashSet;
use std::hash::Hash;

pub trait IntoPowerSet<T> {
  fn powerset(&self) -> PowerSet<T>;
}

impl<T: Clone + Eq + Hash> IntoPowerSet<T> for HashSet<T> {
  fn powerset(&self) -> PowerSet<T> {
    PowerSet::new(self)
  }
}

pub struct PowerSet<T> {
  items: Vec<(bool, T)>,
  done: bool,
}

impl<T: Clone + Eq + Hash> PowerSet<T> {
  fn new(s: &HashSet<T>) -> Self {
    Self {
      items: s.iter().map(|i| (false, i.clone())).collect(),
      done: false,
    }
  }
}

impl<T: Clone + Eq + Hash> Iterator for PowerSet<T> {
  type Item = HashSet<T>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.done {
      return None;
    }
    let mut carrying = true;
    let mut sub = HashSet::new();

    for mut i in self.items.iter_mut() {
      if i.0 {
        sub.insert(i.1.clone());
      }

      if carrying {
        if i.0 {
          i.0 = false;
        } else {
          i.0 = true;
          carrying = false;
        }
      }
    }

    if carrying {
      self.done = true
    }
    Some(sub)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_powerset() {
    let set = [1, 2, 3].into_iter().collect::<HashSet<_>>();
    assert_eq!(set.powerset().count(), 8);

    let set = [1, 2, 3, 4].into_iter().collect::<HashSet<_>>();
    assert_eq!(set.powerset().count(), 16);
  }
}
