const NULL: usize = !0;

use slab::Slab;
use std::collections::VecDeque;

#[derive(Debug)]
struct Node<T> {
    parent: usize,
    children: [usize; 2],
    key: T,
    red: bool,
}

impl<T> Node<T> {
    fn new(key: T, nil_sentinel: usize) -> Node<T> {
        Node {
            parent: nil_sentinel,
            children: [nil_sentinel, nil_sentinel],
            key: key,
            red: false,
        }
    }
}

pub struct RedBlack<T> {
    slab: Slab<Node<T>>,
    root: usize,
    nil_sentinel: usize,
}

impl<T> RedBlack<T> where
T: std::default::Default + std::cmp::PartialOrd + std::fmt::Debug + Copy {
    pub fn new() -> RedBlack<T> {
        let mut rb = RedBlack {
            slab: Slab::new(),
            root: NULL,
            nil_sentinel: NULL,
        };
        let nil_sentinel = rb.slab.insert(Node::new(T::default(), NULL));
        rb.nil_sentinel = nil_sentinel;
        rb.root = nil_sentinel;
        rb
    }

    fn rotate(&mut self, x: usize, dir: usize) {
        let y = self.slab[x].children[dir^1];
        self.slab[x].children[dir^1] = self.slab[y].children[dir];
        let y_chld = self.slab[y].children[dir];
        if y_chld != self.nil_sentinel {
            self.slab[y_chld].parent = x;
        }
        self.slab[y].parent = self.slab[x].parent;
        let x_parent = self.slab[x].parent;
        if x_parent == self.nil_sentinel {
            self.root = y;
        } else {
            let sib_dir = if self.slab[x_parent].children[0] == x { 0 } else { 1 };
            self.slab[x_parent].children[sib_dir] = y;
        }
        self.slab[y].children[dir] = x;
        self.slab[x].parent = y;
    }

    pub fn insert(&mut self, key: T) {
        let z = self.slab.insert(Node::new(key, self.nil_sentinel));

        // empty insert - root is black
        if self.root == self.nil_sentinel {
            self.root = z;
            self.slab[self.root].parent = self.nil_sentinel;
            self.slab[z].red = false;
            return;
        }

        self.slab[z].red = true;
        // find the appropriate spot to put a red leaf
        let mut curr = self.root;
        loop {
            let direction = if self.slab[curr].key < self.slab[z].key { 1 } else { 0 };
            let next = self.slab[curr].children[direction];
            if next == self.nil_sentinel {
                self.slab[curr].children[direction] = z;
                self.slab[z].parent = curr;
                break;
            }
            curr = next;
        }

        self.red_black_fixup(z);
    }

    fn red_black_fixup(&mut self, z_param: usize) {
        // blacken the root
        self.slab[self.root].red = false;
    }

    pub fn search(&mut self, key: T) -> Option<T> {
        let mut curr = self.root;

        while curr != self.nil_sentinel {
            if self.slab[curr].key == key {
                return Some(key);
            }
            let direction = if self.slab[curr].key < key { 1 } else { 0 };
            curr = self.slab[curr].children[direction];
        }
        None
    }

    fn verify_black_height(&self, x: usize) -> i32 {
        if x == self.nil_sentinel {
            return 0;
        }
        let left_height = self.verify_black_height(self.slab[x].children[0]);
        let right_height = self.verify_black_height(self.slab[x].children[1]);

        assert!(left_height != -1 && right_height != -1 && left_height == right_height, "red-black properties have been violated!");

        let add = if self.slab[x].red { 0 } else { 1 };
        return left_height + add;
    }

    fn verify_children_color(&self) -> bool {
        if self.root == self.nil_sentinel {
            return true;
        }
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_front(self.root);

        while !queue.is_empty() {
            let curr = queue.pop_front().unwrap();
            if curr == NULL {
                break;
            }

            let left_child = self.slab[curr].children[0];
            if left_child != NULL {
                assert!(!self.slab[curr].red || (self.slab[curr].red && !self.slab[left_child].red)); // red node, non-red child
                queue.push_back(left_child);
            }
            let right_child = self.slab[curr].children[1];
            if right_child != NULL {
                assert!(!self.slab[curr].red || (self.slab[curr].red && !self.slab[right_child].red)); // red node, non-red child
                queue.push_back(left_child);
            }
        }

        return true;
    }

    /*
     * properties
     * - root property: root is black
     * - leaf nodes (NULL) are black (pointless here given my sentinel is a NULL, not a real node)
     * - red property: children of a red node are black
     * - simple path from node to descendant leaf contains same number of black nodes
     */
    fn is_valid(&self) {
        assert!(!self.slab[self.root].red); // root is black
        self.verify_children_color();
        self.verify_black_height(self.root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert() {
        let mut rb: RedBlack<i32> = RedBlack::new();

        rb.insert(5);
        println!("insert 1 succeeded");

        rb.insert(6);
        println!("insert 2 succeeded");

        rb.insert(7);
        println!("insert 3 succeeded");

        assert_eq!(rb.search(5), Some(5));
        assert_eq!(rb.search(6), Some(6));
        assert_eq!(rb.search(7), Some(7));

        rb.is_valid(); // will panic if it must
    }

    #[test]
    fn test_basic_rotation() {
        let mut rb: RedBlack<i32> = RedBlack::new();

        rb.insert(5); // x
        rb.insert(1); // alpha
        rb.insert(8); // y
        rb.insert(7); // beta
        rb.insert(9); // gamma

        /*
         *      x
         *     / \
         *    /   y
         *   a   / \
         *      b   g
         */

        assert_eq!(rb.slab[1].key, 5);
        assert_eq!(rb.slab[1].parent, rb.nil_sentinel);
        assert_eq!(rb.slab[1].children[0], 2); // x's left points to 2 in the slab i.e. alpha
        assert_eq!(rb.slab[1].children[1], 3); // x's right points to 3 in the slab i.e. y

        assert_eq!(rb.slab[2].key, 1);
        assert_eq!(rb.slab[2].parent, 1);
        assert_eq!(rb.slab[2].children[0], rb.nil_sentinel);
        assert_eq!(rb.slab[2].children[1], rb.nil_sentinel);

        assert_eq!(rb.slab[3].key, 8);
        assert_eq!(rb.slab[3].parent, 1); 
        assert_eq!(rb.slab[3].children[0], 4); // y's left points to 4 in the slab i.e. beta
        assert_eq!(rb.slab[3].children[1], 5); // y's right points to 5 in the slab i.e. gamma

        assert_eq!(rb.slab[4].key, 7);
        assert_eq!(rb.slab[4].parent, 3);
        assert_eq!(rb.slab[4].children[0], rb.nil_sentinel);
        assert_eq!(rb.slab[4].children[1], rb.nil_sentinel);
        assert_eq!(rb.slab[5].key, 9);
        assert_eq!(rb.slab[5].parent, 3);
        assert_eq!(rb.slab[5].children[0], rb.nil_sentinel);
        assert_eq!(rb.slab[5].children[1], rb.nil_sentinel);

        rb.rotate(1, 0); // left-rotate x

        /*
         *      y
         *     / \
         *    x   g
         *   / \
         *  a   b
         */

        // slab entries should be the same, but their links should reflect the new tree topology

        assert_eq!(rb.slab[1].key, 5);
        assert_eq!(rb.slab[2].key, 1);
        assert_eq!(rb.slab[1].parent, 3); // x's new parent is y
        assert_eq!(rb.slab[3].children[0], 1); // y's left child is x
        assert_eq!(rb.slab[3].children[1], 5); // y's right child is gamma
        assert_eq!(rb.slab[5].key, 9);
        assert_eq!(rb.slab[5].parent, 3);
        assert_eq!(rb.slab[1].children[0], 2); // x's left child is alpha
        assert_eq!(rb.slab[1].children[1], 4); // x's right child is beta
        assert_eq!(rb.slab[2].parent, 1); // alpha's parent is x
        assert_eq!(rb.slab[4].parent, 1); // beta's parent is x

        rb.rotate(3, 1); // right-rotate y brings our tree back to the original

        assert_eq!(rb.slab[1].key, 5);
        assert_eq!(rb.slab[1].parent, rb.nil_sentinel);
        assert_eq!(rb.slab[1].children[0], 2); // x's left points to 2 in the slab i.e. alpha
        assert_eq!(rb.slab[1].children[1], 3); // x's right points to 3 in the slab i.e. y

        assert_eq!(rb.slab[2].key, 1);
        assert_eq!(rb.slab[2].parent, 1);
        assert_eq!(rb.slab[2].children[0], rb.nil_sentinel);
        assert_eq!(rb.slab[2].children[1], rb.nil_sentinel);

        assert_eq!(rb.slab[3].key, 8);
        assert_eq!(rb.slab[3].parent, 1); 
        assert_eq!(rb.slab[3].children[0], 4); // y's left points to 4 in the slab i.e. beta
        assert_eq!(rb.slab[3].children[1], 5); // y's right points to 5 in the slab i.e. gamma

        assert_eq!(rb.slab[4].key, 7);
        assert_eq!(rb.slab[4].parent, 3);
        assert_eq!(rb.slab[4].children[0], rb.nil_sentinel);
        assert_eq!(rb.slab[4].children[1], rb.nil_sentinel);
        assert_eq!(rb.slab[5].key, 9);
        assert_eq!(rb.slab[5].parent, 3);
        assert_eq!(rb.slab[5].children[0], rb.nil_sentinel);
        assert_eq!(rb.slab[5].children[1], rb.nil_sentinel);
    }

    #[test]
    fn test_many_insert() {
        let mut num = 1u32;
        let mut rb: RedBlack<u32> = RedBlack::new();

        for _ in 0..1000000 {
            num = num.wrapping_mul(17).wrapping_add(255);
            rb.insert(num);
        }

        rb.is_valid(); // will panic if it must
    }
}
