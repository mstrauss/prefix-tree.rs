#[derive(Debug)]
pub struct Node<T> {
    key: Vec<u32>,
    pub value: Option<T>,
    child: Option<Box<Node<T>>>,
    sibling: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new<K: Into<Vec<u32>>>(key: K, value: T) -> Node<T> {
        Node {
            key: key.into(),
            value: Some(value),
            child: None,
            sibling: None,
        }
    }

    fn boxed<K: Into<Vec<u32>>>(key: K, value: T) -> Box<Node<T>> {
        Box::new(Self::new(key, value))
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

    pub fn find_mut<K: AsRef<[u32]>>(&mut self, key: K) -> Option<&mut Node<T>> {
        let key = key.as_ref();
        let prefix = self.common_prefix(key);
        if prefix == 0 {
            self.sibling.as_mut().and_then(|x| x.find_mut(key))
        } else if prefix == self.key.len() {
            if prefix == key.len() {
                Some(self)
            } else {
                self.child.as_mut().and_then(|x| x.find_mut(&key[prefix..]))
            }
        } else {
            None
        }
    }

    pub fn insert<K: AsRef<[u32]>>(&mut self, key: K, value: T) {
        let key = key.as_ref();
        let prefix = self.common_prefix(key);
        if prefix == 0 {
            match self.sibling {
                Some(ref mut sibling) => sibling.insert(key, value),
                _ => self.sibling = Some(Self::boxed(key, value)),
            }
        } else if prefix < key.len() {
            if prefix < self.key.len() {
                self.child = Some(Box::new(Node {
                    key: self.key.split_off(prefix),
                    value: self.value.take(),
                    child: self.child.take(),
                    sibling: None,
                }));
                self.key.shrink_to_fit()
            }
            match self.child {
                Some(ref mut child) => child.insert(&key[prefix..], value),
                _ => self.child = Some(Self::boxed(&key[prefix..], value)),
            }
        }
    }
}

impl Node<u32> {
    pub fn insert_and_count<K: AsRef<[u32]>>(&mut self, key: K) {
        let key = key.as_ref();
        let prefix = self.common_prefix(key);
        if prefix == 0 {
            match self.sibling {
                Some(ref mut sibling) => sibling.insert_and_count(key),
                _ => self.sibling = Some(Self::boxed(key, 1u32)),
            }
        } else if prefix < key.len() {
            let mut old_count = 1u32;
            if prefix < self.key.len() {
                old_count = self.value.unwrap();
                self.child = Some(Box::new(Node {
                    key: self.key.split_off(prefix),
                    value: self.value.take(),
                    child: self.child.take(),
                    sibling: None,
                }));
                self.key.shrink_to_fit()
            }
            match self.child {
                Some(ref mut child) => child.insert_and_count(&key[prefix..]),
                _ => self.child = Some(Self::boxed(&key[prefix..], 1u32)),
            }
            // update self-count
            self.value = Some(self.value.unwrap_or(old_count) + 1u32);
        } else {
            // same node!  increment count
            match self.value {
                Some(count) => self.value = Some(count + 1u32),
                _ => panic!("self.value may not be missing"),
            }
        }
    }
}

#[derive(Debug)]
pub struct Tree<T> {
    root: Option<Box<Node<T>>>,
}

impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree {
            root: None,
        }
    }

    pub fn find<K: AsRef<[u32]>>(&self, key: K) -> Option<&Node<T>> {
        self.root.as_ref().and_then(|x| x.find(key))
    }

    pub fn find_mut<K: AsRef<[u32]>>(&mut self, key: K) -> Option<&mut Node<T>> {
        self.root.as_mut().and_then(|x| x.find_mut(key))
    }

    fn insert<K: AsRef<[u32]>>(&mut self, key: K, value: T) {
        match self.root {
            Some(ref mut root) => root.insert(key, value),
            _ => self.root = Some(Node::boxed(key.as_ref(), value))
        }
    }
}

impl Tree<u32> {
    pub fn insert_and_count<K: AsRef<[u32]>>(&mut self, key: K) {
        // inserting with auto-value
        match self.root {
            Some(ref mut root) => root.insert_and_count(key),
            _ => self.root = Some(Node::boxed(key.as_ref(), 1u32))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Tree};

    #[test]
    fn test_common_prefix_empty() {
        assert!(Node::new(vec![3u32, 137u32, 2u32], ()).common_prefix([]) == 0);
    }

    #[test]
    fn test_common_prefix_short() {
        assert!(Node::new(vec![3u32, 137u32, 2u32], ()).common_prefix(vec![3u32, 137u32, 8u32, 2u32]) == 2);
    }

    // #[test]
    // fn test_common_prefix_bytes() {
    //     let left = "foó";   // [b'f', b'o', b'\xc3', b'\xb3']
    //     let right = "foò";  // [b'f', b'o', b'\xc3', b'\xb2']
    //     assert!(Node::new(left, ()).common_prefix(right) == 3);
    // }

    #[test]
    fn test_find_empty() {
        let t = Tree::<()> { root: None };
        assert!(t.find([]).is_none());
        assert!(t.find(vec![3u32, 137u32, 2u32]).is_none());
    }

    #[test]
    fn test_find_mut_empty() {
        let mut t = Tree::<()> { root: None };
        assert!(t.find_mut([]).is_none());
        assert!(t.find_mut(vec![3u32, 137u32, 2u32]).is_none());
    }

    fn sample_tree() -> Tree<u32> {
        Tree {
            root: Some(Box::new(Node {
                key: vec![3u32, 137u32],
                value: Some(0),
                child: Some(Box::new(Node {
                    key: vec![137u32],
                    value: Some(1),
                    child: None,
                    sibling: Some(Node::boxed(vec![0u32], 2))
                })),
                sibling: Some(Node::boxed(vec![1u32, 2u32, 9u32], 3)),
            })),
        }
    }

    #[test]
    fn test_find_simple() {
        assert!(sample_tree().find(vec![3u32, 137u32]).unwrap().value == Some(0));
    }

    #[test]
    fn test_find_mut_simple() {
        assert!(sample_tree().find_mut(vec![3u32, 137u32]).unwrap().value == Some(0));
    }

    #[test]
    fn test_find_child() {
        assert!(sample_tree().find(vec![3u32, 137u32, 137u32]).unwrap().value == Some(1));
    }

    #[test]
    fn test_find_mut_child() {
        assert!(sample_tree().find_mut(vec![3u32, 137u32, 137u32]).unwrap().value == Some(1));
    }

    #[test]
    fn test_find_sibling() {
        assert!(sample_tree().find(vec![1u32, 2u32, 9u32]).unwrap().value == Some(3));
    }

    #[test]
    fn test_find_mut_sibling() {
        assert!(sample_tree().find_mut(vec![1u32, 2u32, 9u32]).unwrap().value == Some(3));
    }

    #[test]
    fn test_find_missing() {
        assert!(sample_tree().find(vec![999u32]).is_none());
    }

    #[test]
    fn test_find_mut_missing() {
        assert!(sample_tree().find_mut(vec![999u32]).is_none());
    }

    #[test]
    fn test_find_shorter() {
        assert!(sample_tree().find(vec![3u32]).is_none());
    }

    #[test]
    fn test_find_mut_shorter() {
        assert!(sample_tree().find_mut(vec![3u32]).is_none());
    }

    #[test]
    fn test_find_longer() {
        assert!(sample_tree().find(vec![3u32, 137u32, 137u32, 137u32]).is_none());
    }

    #[test]
    fn test_find_mut_longer() {
        assert!(sample_tree().find_mut(vec![3u32, 137u32, 137u32, 137u32]).is_none());
    }

    #[test]
    fn test_insert_empty() {
        let mut t = Tree::new();
        t.insert(vec![999u32], ());
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![999u32]);
        assert!(root.value == Some(()));
        assert!(root.child.is_none());
        assert!(root.sibling.is_none());
    }

    #[test]
    fn test_insert_append() {
        let mut t = Tree::new();
        t.insert(vec![3u32], 0);
        t.insert(vec![3u32, 137u32], 1);
        t.insert(vec![3u32, 137u32, 2u32], 2);
        let foo = t.root.as_ref().unwrap();
        assert!(foo.key == vec![3u32]);
        assert!(foo.value == Some(0));
        assert!(foo.sibling.is_none());
        let bar = foo.child.as_ref().unwrap();
        assert!(bar.key == vec![137u32]);
        assert!(bar.value == Some(1));
        assert!(bar.sibling.is_none());
        let baz = bar.child.as_ref().unwrap();
        assert!(baz.key == vec![2u32]);
        assert!(baz.value == Some(2));
        assert!(baz.child.is_none());
        assert!(baz.sibling.is_none());
    }

    #[test]
    fn test_insert_sibling() {
        let mut t = Tree::new();
        t.insert(vec![987u32], 0);
        t.insert(vec![654u32], 1);
        t.insert(vec![321u32], 2);
        let foo = t.root.as_ref().unwrap();
        assert!(foo.key == vec![987u32]);
        assert!(foo.value == Some(0));
        assert!(foo.child.is_none());
        let bar = foo.sibling.as_ref().unwrap();
        assert!(bar.key == vec![654u32]);
        assert!(bar.value == Some(1));
        assert!(bar.child.is_none());
        let quux = bar.sibling.as_ref().unwrap();
        assert!(quux.key == vec![321u32]);
        assert!(quux.value == Some(2));
        assert!(quux.child.is_none());
        assert!(quux.sibling.is_none());
    }

    #[test]
    fn test_insert_split() {
        let mut t = Tree::new();
        t.insert(vec![3u32, 137u32, 2u32], 0);
        t.insert(vec![3u32, 137u32, 99u32, 22u32], 1);
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![3u32, 137u32]);
        assert!(root.value.is_none());
        assert!(root.sibling.is_none());
        let foo = root.child.as_ref().unwrap();
        assert!(foo.key == vec![2u32]);
        assert!(foo.value == Some(0));
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
    fn test_insert_and_count() {
        let mut t = sample_tree();
        t.insert_and_count(vec![3u32, 137u32, 137u32, 999u32]);
        println!("{:?}", t);
    }

    #[test]
    fn test_insert_empty_auto_counting() {
        let mut t = Tree::new();
        t.insert_and_count(vec![999u32]);
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![999u32]);
        assert!(root.value == Some(1));
        assert!(root.child.is_none());
        assert!(root.sibling.is_none());
    }

    #[test]
    fn test_insert_append_auto_counting() {
        let mut t = Tree::new();
        t.insert_and_count(vec![3u32]);
        t.insert_and_count(vec![3u32, 137u32]);
        t.insert_and_count(vec![3u32, 137u32, 2u32]);
        println!("after third: {:?}", t);
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
    fn test_insert_sibling_auto_counting() {
        let mut t = Tree::new();
        t.insert_and_count(vec![987u32]);
        t.insert_and_count(vec![654u32]);
        t.insert_and_count(vec![321u32]);
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
    fn test_insert_split_auto_counting() {
        let mut t = Tree::new();
        t.insert_and_count(vec![3u32, 137u32, 2u32]);
        println!("after first: {:?}", t);
        t.insert_and_count(vec![3u32, 137u32, 99u32, 2u32]);
        println!("after second: {:?}", t);
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![3u32, 137u32]);
        assert!(root.value == Some(2));
        assert!(root.sibling.is_none());
        let foo = root.child.as_ref().unwrap();
        assert!(foo.key == vec![2u32]);
        assert!(foo.value == Some(1));
        assert!(foo.child.is_none());
        let bar = foo.sibling.as_ref().unwrap();
        assert!(bar.key == vec![99u32, 2u32]);
        assert!(bar.value == Some(1));
        assert!(bar.sibling.is_none());
        assert!(bar.child.is_none());
    }

    #[test]
    fn test_insert_twice_auto_counting() {
        let mut t = Tree::new();
        t.insert_and_count(vec![3u32, 137u32, 2u32]);
        t.insert_and_count(vec![3u32, 137u32, 2u32]);
        let root = t.root.as_ref().unwrap();
        assert!(root.key == vec![3u32, 137u32, 2u32]);
        assert!(root.value == Some(2));
        assert!(root.sibling.is_none());
    }

    fn sample_apriori_tree() -> Tree<u32> {
        let mut t: Tree<u32> = Tree::new();
        // total counts are (ordered desc.) [all input vecs in this order]
        // 8: 8 times, 6: 5 times, 2: 5 times, 9: 4 times, 5: 4 times,
        // 4: 4 times, 1: 4 times, 0: 4 times, 7: 3 times, 3: 2 times
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![8, 5, 1, 3]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![6, 2, 4, 7]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![8, 6, 2, 5, 4, 1]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![2, 8, 4, 0, 7]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![8, 6, 2, 0]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![6, 8, 4, 1]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![8, 5, 0]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![8, 6, 5, 0, 3]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![8, 2]);
        println!("Building Apriori sample tree:\n{:?}", t);
        t.insert_and_count(vec![1, 7]);
        println!("Building Apriori sample tree:\n{:?}", t);
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
