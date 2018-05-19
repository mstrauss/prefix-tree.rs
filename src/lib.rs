use std::rc::Rc;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
#[derive(PartialEq, Eq, Hash)]
pub struct Node<T> {
    key: Vec<u32>,
    pub value: Option<T>,
    child: Option<Rc<Node<T>>>,
    sibling: Option<Rc<Node<T>>>,
    next: Option<Rc<Node<T>>>,
    tree: *mut Tree,
}

impl<T> Node<T> {
    pub fn new<K: Into<Vec<u32>>>(key: K, value: T, tree: *mut Tree) -> Node<T> {
        Node {
            key: key.into(),
            value: Some(value),
            child: None,
            sibling: None,
            next: None,
            tree: tree,
        }
    }

    fn common_prefix<K: AsRef<[u32]>>(&self, other: K) -> usize {
        self.key.iter()
            .zip(other.as_ref().into_iter())
            .take_while(|&(a, b)| a == b)
            .count()
    }

    pub fn find<K: AsRef<[u32]>>(&self, key: K) -> Option<&Node<T>> {
        let key = key.as_ref();
        let prefix = self.common_prefix(key);
        if prefix == 0 {
            self.sibling.as_ref().and_then(|x| x.find(key))
        } else if prefix == self.key.len() {
            if prefix == key.len() {
                Some(self)
            } else {
                self.child.as_ref().and_then(|x| x.find(&key[prefix..]))
            }
        } else {
            None
        }
    }
}

enum AppendType {
    SameNode,
    NewStraightChild,
    NewGayChild,
    NewSibling,
}

impl Node<u32> {
    fn boxed<K: Into<Vec<u32>>>(key: K, value: u32, tree: *mut Tree) -> Rc<Node<u32>> {
        let n = Rc::new(Self::new(key, value, tree));
        unsafe { (*tree).index_node(&n) };
        n
    }

    pub fn append<K: AsRef<[u32]>>(&self, key: K) -> Node<u32> {
        let key = key.as_ref();
        let prefix = self.common_prefix(key);
        let state;
        if prefix == 0 {
            state = AppendType::NewSibling;
        } else if prefix < key.len() {
            if prefix < self.key.len() {
                state = AppendType::NewGayChild;
            } else {
                state = AppendType::NewStraightChild;
            }
        } else {
            state = AppendType::SameNode;
        }

        Node {
            key: match state {
                AppendType::NewGayChild => self.key[0..prefix].to_vec(),
                _ => self.key.clone(),
            },
            value: match state {
                AppendType::NewSibling => self.value.clone(),
                _ => Some(self.value.unwrap() + 1u32),
            },
            child: match state {
                AppendType::NewGayChild => Some(Rc::new(Node {
                    key: self.key[prefix..].to_vec(),
                    value: self.value.clone(),
                    child: self.child.clone(),
                    sibling: None,
                    next: None,
                    tree: self.tree,
                }.append(&key[prefix..]))),
                AppendType::NewStraightChild => match self.child {
                    Some(ref child) => Some(Rc::new(child.append(&key[prefix..]))),
                    _ => Some(Self::boxed(&key[prefix..], 1u32, self.tree)),
                },
                _ => self.child.clone(),
            },
            sibling: match prefix {
                0 => match self.sibling {
                    Some(ref sibling) => Some(Rc::new(sibling.append(key))),
                    _ => Some(Self::boxed(key, 1u32, self.tree)),
                },
                _ => self.sibling.clone(),
            },
            next: None,
            tree: self.tree,
        }
    }
}

#[derive(Debug)]
pub struct Tree {
    root: Option<Rc<Node<u32>>>,
    nodeindex: HashMap<u32, HashSet<Rc<Node<u32>>>>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            root: None,
            nodeindex: HashMap::new(),
        }
    }

    pub fn index_node(&mut self, node: &Rc<Node<u32>>) {
        let ref key = node.key;
        for k in key {
            let nodeindex = &mut self.nodeindex;
            if !nodeindex.contains_key(k) {
                nodeindex.insert(k.clone(), HashSet::new());
            }
            nodeindex.get_mut(k).unwrap().insert(Rc::clone(node));
        }
    }

    pub fn find<K: AsRef<[u32]>>(&self, key: K) -> Option<&Node<u32>> {
        self.root.as_ref().and_then(|x| x.find(key))
    }

    pub fn append<K: AsRef<[u32]>>(&mut self, key: K) {
        self.root = match self.root {
            Some(ref root) => Some(Rc::new(root.append(key))),
            _ => Some(Node::boxed(key.as_ref(), 1u32, self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Tree};
    use std::ptr;
    use std::collections::HashSet;

    #[test]
    fn test_common_prefix_empty() {
        assert!(Node::new(vec![3u32, 137u32, 2u32], (), ptr::null_mut()).common_prefix([]) == 0);
    }

    #[test]
    fn test_common_prefix_short() {
        assert!(Node::new(vec![3u32, 137u32, 2u32], (), ptr::null_mut()).common_prefix(vec![3u32, 137u32, 8u32, 2u32]) == 2);
    }

    #[test]
    fn test_find_empty() {
        let t = Tree::new();
        assert!(t.find([]).is_none());
        assert!(t.find(vec![3u32, 137u32, 2u32]).is_none());
    }

    fn sample_tree() -> Tree {
        let mut t = Tree::new();
        t.append(vec![3u32, 137u32]);
        t.append(vec![3u32, 137u32, 137u32]);
        t.append(vec![1u32, 2u32, 9u32]);
        t
    }

    #[test]
    fn test_sample_tree_nodeindex() {
        let ref t = sample_tree();
        let ref ni = t.nodeindex;
        println!("node index: {:?}", ni);
        assert!(ni.len() == 5);
        let nodes_3 = ni.get(&3).unwrap();
        assert!(nodes_3.len() == 1);
        let n1 = t.find(vec![3u32, 137u32]).unwrap();
        println!("n1: {:?}", n1);
        // assert!(nodes_3.contains(&*n1));
        // assert!(false);
    }

    #[test]
    fn test_find_simple() {
        assert!(sample_tree().find(vec![3u32, 137u32]).unwrap().value == Some(2));
    }

    #[test]
    fn test_find_child() {
        assert!(sample_tree().find(vec![3u32, 137u32, 137u32]).unwrap().value == Some(1));
    }

    #[test]
    fn test_find_sibling() {
        assert!(sample_tree().find(vec![1u32, 2u32, 9u32]).unwrap().value == Some(1));
    }

    #[test]
    fn test_find_missing() {
        assert!(sample_tree().find(vec![999u32]).is_none());
    }

    #[test]
    fn test_find_shorter() {
        assert!(sample_tree().find(vec![3u32]).is_none());
    }

     #[test]
    fn test_find_longer() {
        assert!(sample_tree().find(vec![3u32, 137u32, 137u32, 137u32]).is_none());
    }

    #[test]
    fn test_insert_empty() {
        let mut t = Tree::new();
        t.append(vec![999u32]);
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![999u32]);
        assert!(root.value == Some(1));
        assert!(root.child.is_none());
        assert!(root.sibling.is_none());
    }

    #[test]
    fn test_insert_append() {
        let mut t = Tree::new();
        t.append(vec![3u32]);
        t.append(vec![3u32, 137u32]);
        t.append(vec![3u32, 137u32, 2u32]);
        let foo = t.root.as_ref().unwrap();
        assert!(foo.key == vec![3u32]);
        assert!(foo.value == Some(3));
        assert!(foo.sibling.is_none());
        let bar = foo.child.as_ref().unwrap();
        assert!(bar.key == vec![137u32]);
        assert!(bar.value == Some(2));
        assert!(bar.sibling.is_none());
        let baz = bar.child.as_ref().unwrap();
        assert!(baz.key == vec![2u32]);
        assert!(baz.value == Some(1));
        assert!(baz.child.is_none());
        assert!(baz.sibling.is_none());
    }

    #[test]
    fn test_insert_sibling() {
        let mut t = Tree::new();
        t.append(vec![987u32]);
        t.append(vec![654u32]);
        t.append(vec![321u32]);
        let foo = t.root.as_ref().unwrap();
        assert!(foo.key == vec![987u32]);
        assert!(foo.value == Some(1));
        assert!(foo.child.is_none());
        let bar = foo.sibling.as_ref().unwrap();
        assert!(bar.key == vec![654u32]);
        assert!(bar.value == Some(1));
        assert!(bar.child.is_none());
        let quux = bar.sibling.as_ref().unwrap();
        assert!(quux.key == vec![321u32]);
        assert!(quux.value == Some(1));
        assert!(quux.child.is_none());
        assert!(quux.sibling.is_none());
    }

    #[test]
    fn test_insert_split() {
        let mut t = Tree::new();
        t.append(vec![3u32, 137u32, 2u32]);
        println!("test_insert_split/pre: {:?}", t);
        t.append(vec![3u32, 137u32, 99u32, 22u32]);
        println!("test_insert_split/post: {:?}", t);
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![3u32, 137u32]);
        assert!(root.value == Some(2));
        assert!(root.sibling.is_none());
        let foo = root.child.as_ref().unwrap();
        assert!(foo.key == vec![2u32]);
        assert!(foo.value == Some(1));
        assert!(foo.child.is_none());
        let bar = foo.sibling.as_ref().unwrap();
        assert!(bar.key == vec![99u32, 22u32]);
        assert!(bar.value == Some(1));
        assert!(bar.sibling.is_none());
        assert!(bar.child.is_none());
    }

    #[test]
    fn test_fmt_debug() {
        println!("{:?}", sample_tree());
    }

    #[test]
    fn test_insert_twice() {
        let mut t = Tree::new();
        t.append(vec![3u32, 137u32, 2u32]);
        t.append(vec![3u32, 137u32, 2u32]);
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![3u32, 137u32, 2u32]);
        assert!(root.value == Some(2));
        assert!(root.sibling.is_none());
    }

    fn sample_apriori_tree() -> Tree {
        let mut t: Tree = Tree::new();
        // total counts are (ordered desc.) [all input vecs in this order]
        // 8: 8 times, 6: 5 times, 2: 5 times, 9: 4 times, 5: 4 times,
        // 4: 4 times, 1: 4 times, 0: 4 times, 7: 3 times, 3: 2 times
        println!("NEW Apriori sample tree:\n{:?}", t);
        t.append(vec![8, 5, 1, 3]);
        println!("+ [8, 5, 1, 3] => {:?}", t);
        t.append(vec![6, 2, 4, 7]);
        println!("+ [6, 2, 4, 7] => {:?}", t);
        t.append(vec![8, 6, 2, 5, 4, 1]);
        println!("+ [8, 6, 2, 5, 4, 1] => {:?}", t);
        t.append(vec![2, 8, 4, 0, 7]);
        println!("+ [2, 8, 4, 0, 7] => {:?}", t);
        t.append(vec![8, 6, 2, 0]);
        println!("+ [8, 6, 2, 0] => {:?}", t);
        t.append(vec![6, 8, 4, 1]);
        println!("+ [6, 8, 4, 1] => {:?}", t);
        t.append(vec![8, 5, 0]);
        println!("+ [8, 5, 0] => {:?}", t);
        t.append(vec![8, 6, 5, 0, 3]);
        println!("+ [8, 6, 5, 0, 3] => {:?}", t);
        t.append(vec![8, 2]);
        println!("+ [8, 2] => {:?}", t);
        t.append(vec![1, 7]);
        println!("+ [1, 7] => {:?}", t);
        t
    }

    #[test]
    fn test_sample_apriori_tree() {
        let t = sample_apriori_tree();
        println!("Apriori sample:\n{:?}", t);

        let root = t.root.as_ref().unwrap();
        assert_eq!(root.key, vec![8]);
        assert_eq!(root.value, Some(6));
        // child below
        // sibling below

        let r_5 = root.child.as_ref().unwrap();
        assert_eq!(r_5.key, vec![5]);
        assert_eq!(r_5.value, Some(2));
        // child, sibling below

        let r_5_1_3 = r_5.child.as_ref().unwrap();
        assert_eq!(r_5_1_3.key, vec![1, 3]);
        assert_eq!(r_5_1_3.value, Some(1));
        assert!(r_5_1_3.child.is_none());
        // sibling below

        let r_5_0 = r_5_1_3.sibling.as_ref().unwrap();
        assert_eq!(r_5_0.key, vec![0]);
        assert_eq!(r_5_0.value, Some(1));
        assert!(r_5_0.child.is_none());
        assert!(r_5_0.sibling.is_none());

        let r_6 = r_5.sibling.as_ref().unwrap();
        assert_eq!(r_6.key, vec![6]);
        assert_eq!(r_6.value, Some(3));
        // child below
        // sibling below

        let r_6_2 = r_6.child.as_ref().unwrap();
        assert_eq!(r_6_2.key, vec![2]);
        assert_eq!(r_6_2.value, Some(2));
        // child below
        // sibling below

        let r_6_2_5_4_1 = r_6_2.child.as_ref().unwrap();
        assert_eq!(r_6_2_5_4_1.key, vec![5, 4, 1]);
        assert_eq!(r_6_2_5_4_1.value, Some(1));
        assert!(r_6_2_5_4_1.child.is_none());
        // sibling below

        let r_6_2_0 = r_6_2_5_4_1.sibling.as_ref().unwrap();
        assert_eq!(r_6_2_0.key, vec![0]);
        assert_eq!(r_6_2_0.value, Some(1));
        assert!(r_6_2_0.child.is_none());
        assert!(r_6_2_0.sibling.is_none());

        let r_2 = r_6.sibling.as_ref().unwrap();
        assert_eq!(r_2.key, vec![2]);
        assert_eq!(r_2.value, Some(1));
        assert!(r_2.child.is_none());
        assert!(r_2.sibling.is_none());

        let rs_6 = root.sibling.as_ref().unwrap();
        assert_eq!(rs_6.key, vec![6]);
        assert_eq!(rs_6.value, Some(2));
        // child below
        // sibling below

        let rs_6_2_4_7 = rs_6.child.as_ref().unwrap();
        assert_eq!(rs_6_2_4_7.key, vec![2, 4, 7]);
        assert_eq!(rs_6_2_4_7.value, Some(1));
        assert!(rs_6_2_4_7.child.is_none());
        // sibling below

        let rs_6_8_4_1 = rs_6_2_4_7.sibling.as_ref().unwrap();
        assert_eq!(rs_6_8_4_1.key, vec![8, 4, 1]);
        assert_eq!(rs_6_8_4_1.value, Some(1));
        assert!(rs_6_8_4_1.child.is_none());
        assert!(rs_6_8_4_1.sibling.is_none());

        let rs_2 = rs_6.sibling.as_ref().unwrap();
        assert_eq!(rs_2.key, vec![2, 8, 4, 0, 7]);
        assert_eq!(rs_2.value, Some(1));
        assert!(rs_2.child.is_none());
        // sibling below

        let rs_1 = rs_2.sibling.as_ref().unwrap();
        assert_eq!(rs_1.key, vec![1, 7]);
        assert_eq!(rs_1.value, Some(1));
        assert!(rs_1.child.is_none());
        assert!(rs_1.sibling.is_none());
    }

}
