use crate::redblack::RedBlack;
use std::{mem, ptr};

#[cfg(test)]
use std::collections::VecDeque;

struct Node<T> {
    parent: *mut Node<T>,
    children: [*mut Node<T>; 2],
    key: T,
    red: bool,
}

fn new_node_ptr<T>(key: T, nil_sentinel: *mut Node<T>) -> *mut Node<T> {
    // use Box to allocate nodes on the heap
    let node = Node::new(key, nil_sentinel);
    Box::into_raw(Box::new(node))
}

impl<T> Node<T> {
    fn new(key: T, nil_sentinel: *mut Node<T>) -> Node<T> {
        Node {
            parent: nil_sentinel,
            children: [nil_sentinel, nil_sentinel],
            key,
            red: false,
        }
    }

    unsafe fn nil_sentinel() -> *mut Node<T> {
        new_node_ptr(
            mem::MaybeUninit::<T>::uninit().assume_init(),
            ptr::null_mut(),
        )
    }
}

pub struct PointerRedBlack<T> {
    root: *mut Node<T>,
    nil_sentinel: *mut Node<T>,
}

impl<T> PointerRedBlack<T>
where
    T: std::cmp::PartialOrd,
{
    unsafe fn rotate(&mut self, x: *mut Node<T>, dir: usize) {
        let y = (*x).children[dir ^ 1];
        (*x).children[dir ^ 1] = (*y).children[dir];
        if (*y).children[dir] != self.nil_sentinel {
            (*(*y).children[dir]).parent = x;
        }
        (*y).parent = (*x).parent;
        if (*x).parent == self.nil_sentinel {
            self.root = y;
        } else {
            let sib_dir = if (*(*x).parent).children[0] == x {
                0
            } else {
                1
            };
            (*(*x).parent).children[sib_dir] = y;
        }
        (*y).children[dir] = x;
        (*x).parent = y;
    }

    unsafe fn tree_minimum(&mut self, mut x: *mut Node<T>) -> *mut Node<T> {
        let mut l = (*x).children[0];
        while l != self.nil_sentinel {
            x = l;
            l = (*x).children[0];
        }
        x
    }

    unsafe fn tree_successor(&mut self, mut x: *mut Node<T>) -> *mut Node<T> {
        if (*x).children[1] != self.nil_sentinel {
            return self.tree_minimum(x);
        }
        let mut y = (*x).parent;
        while y != self.nil_sentinel && x == (*y).children[1] {
            x = y;
            y = (*y).parent;
        }
        y
    }

    unsafe fn insert_fixup(&mut self, mut z: *mut Node<T>) {
        while (*(*z).parent).red {
            let dir = if (*(*(*z).parent).parent).children[0] == (*z).parent {
                1
            } else {
                0
            };

            let y = (*(*(*z).parent).parent).children[dir];

            if (*y).red {
                (*(*z).parent).red = false;
                (*y).red = false;
                (*(*(*z).parent).parent).red = true;
                z = (*(*z).parent).parent;
            } else {
                // y is black, or nil sentinel
                if z == (*(*z).parent).children[dir] {
                    z = (*z).parent;
                    self.rotate(z, dir ^ 1);
                }
                (*(*z).parent).red = false;
                (*(*(*z).parent).parent).red = true;
                self.rotate((*(*z).parent).parent, dir);
            }
        }

        // blacken the root
        (*self.root).red = false;
    }

    unsafe fn delete_fixup(&mut self, mut x: *mut Node<T>) {
        while x != self.root && !(*x).red {
            let dir = if x == (*(*x).parent).children[0] {
                1
            } else {
                0
            };
            let mut w = (*(*x).parent).children[dir];
            if (*w).red {
                (*w).red = false;
                (*(*x).parent).red = true;
                self.rotate((*x).parent, dir ^ 1);
                w = (*(*x).parent).children[dir];
            }
            let wl = (*w).children[0];
            let wr = (*w).children[1];
            if !(*wl).red && !(*wr).red {
                (*w).red = true;
                x = (*x).parent;
            } else {
                let mut wc = (*w).children[dir]; // w child i care about
                let wo = (*w).children[dir ^ 1]; // w other child
                if !(*wc).red {
                    (*wo).red = false;
                    (*w).red = true;
                    self.rotate(w, dir);
                    w = (*(*x).parent).children[dir];

                    // recompute wc after the rotation of w
                    wc = (*w).children[dir];
                }
                (*w).red = (*(*x).parent).red;
                (*(*x).parent).red = false;
                (*wc).red = false;
                self.rotate((*x).parent, dir ^ 1);
                x = self.root
            }
        }

        // blacken x
        (*x).red = false;
    }

    unsafe fn search_(&mut self, key: T) -> Option<*mut Node<T>> {
        let mut curr = self.root;

        while curr != self.nil_sentinel {
            if (*curr).key == key {
                return Some(curr);
            }
            let direction = if (*curr).key < key { 1 } else { 0 };
            curr = (*curr).children[direction];
        }
        None
    }

    #[cfg(test)]
    unsafe fn is_valid(&self) {
        /*
         * properties
         * - root property: root is black
         * - leaf nodes (NULL) are black (pointless here given my sentinel is a NULL, not a real node)
         * - red property: children of a red node are black
         * - simple path from node to descendant leaf contains same number of black nodes
         */
        unsafe fn verify_black_height<T>(rb: &PointerRedBlack<T>, x: *mut Node<T>) -> i32 {
            if x == rb.nil_sentinel {
                return 0;
            }
            let left_height = verify_black_height(rb, (*x).children[0]);
            let right_height = verify_black_height(rb, (*x).children[1]);

            assert!(
                left_height != -1 && right_height != -1 && left_height == right_height,
                "red-black properties have been violated!"
            );

            let add = if (*x).red { 0 } else { 1 };
            left_height + add
        }

        unsafe fn verify_children_color<T>(rb: &PointerRedBlack<T>) -> bool {
            if rb.root == rb.nil_sentinel {
                return true;
            }
            let mut queue: VecDeque<*mut Node<T>> = VecDeque::new();
            queue.push_front(rb.root);

            while !queue.is_empty() {
                let curr = queue.pop_front().unwrap();
                if curr == rb.nil_sentinel {
                    break;
                }

                let l = (*curr).children[0];
                let r = (*curr).children[1];

                // red node must not have red children
                if (*curr).red {
                    assert!(!(*l).red && !(*r).red, "red node has red children");
                }

                if l != rb.nil_sentinel {
                    queue.push_back(l);
                }
                if r != rb.nil_sentinel {
                    queue.push_back(r);
                }
            }
            true
        }

        assert!(!(*self.root).red); // root is black
        verify_children_color(self);
        verify_black_height(self, self.root);
    }
}

impl<T> RedBlack<T> for PointerRedBlack<T>
where
    T: std::cmp::PartialOrd,
{
    fn new() -> PointerRedBlack<T> {
        let mut rb = PointerRedBlack {
            root: ptr::null_mut(),
            nil_sentinel: ptr::null_mut(),
        };

        unsafe {
            let nil_sentinel = Node::nil_sentinel();
            rb.nil_sentinel = nil_sentinel;
            rb.root = nil_sentinel;
        }
        rb
    }

    fn search(&mut self, key: T) -> Option<&T> {
        unsafe {
            if let Some(found_node) = self.search_(key) {
                return Some(&(*found_node).key);
            }
            None
        }
    }

    fn delete(&mut self, key: T) {
        unsafe {
            let z = match self.search_(key) {
                Some(found_node) => found_node,
                None => {
                    return;
                }
            };

            let y =
                if (*z).children[0] == self.nil_sentinel || (*z).children[1] == self.nil_sentinel {
                    z
                } else {
                    self.tree_successor(z)
                };

            let dir = if (*y).children[0] != self.nil_sentinel {
                0
            } else {
                1
            };
            let x = (*y).children[dir];

            let yp = (*y).parent;

            (*x).parent = yp;

            if yp == self.nil_sentinel {
                self.root = x;
            } else {
                let dir = if y == (*yp).children[0] { 0 } else { 1 };
                (*yp).children[dir] = x;
            }

            if y != z {
                mem::swap(&mut (*z).key, &mut (*y).key);
            }
            if !(*y).red {
                self.delete_fixup(x);
            }
        }
    }

    fn insert(&mut self, key: T) {
        unsafe {
            let mut z = new_node_ptr(key, self.nil_sentinel);

            let mut y = self.nil_sentinel;
            let mut x = self.root;

            while x != self.nil_sentinel {
                y = x;
                let dir = if (*z).key < (*x).key { 0 } else { 1 };
                x = (*x).children[dir];
            }

            (*z).parent = y;
            if y == self.nil_sentinel {
                self.root = z;
            } else {
                let dir = if (*z).key < (*y).key { 0 } else { 1 };
                (*y).children[dir] = z;
            }

            (*z).red = true;
            self.insert_fixup(z);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert() {
        let mut rb: PointerRedBlack<i32> = PointerRedBlack::new();

        rb.insert(5);
        rb.insert(6);
        rb.insert(7);

        assert_eq!(rb.search(5), Some(&5));
        assert_eq!(rb.search(6), Some(&6));
        assert_eq!(rb.search(7), Some(&7));

        unsafe {
            rb.is_valid(); // will panic if it must
        }
    }

    #[test]
    fn test_many_insert() {
        let mut num = 1u32;
        let mut rb: PointerRedBlack<u32> = PointerRedBlack::new();

        for _ in 0..1000000 {
            num = num.wrapping_mul(17).wrapping_add(255);
            rb.insert(num);
        }

        unsafe {
            rb.is_valid(); // will panic if it must
        }
    }

    #[test]
    fn test_many_insert_some_delete() {
        let mut rb: PointerRedBlack<i32> = PointerRedBlack::new();

        for i in 500000..1000000 {
            rb.insert(i);
            rb.insert(1000000 - i);
        }

        assert_eq!(rb.search(5), Some(&5));
        assert_eq!(rb.search(50), Some(&50));
        assert_eq!(rb.search(500), Some(&500));
        assert_eq!(rb.search(5000), Some(&5000));
        assert_eq!(rb.search(50000), Some(&50000));
        assert_eq!(rb.search(500000), Some(&500000));

        unsafe {
            rb.is_valid(); // will panic if it must
        }

        rb.delete(5); // the spliced-out node doesn't necessarily have to be the deleted one
        unsafe {
            rb.is_valid(); // will panic if it must
        }
        rb.delete(5);
        assert_eq!(rb.search(5), None);

        rb.delete(50);
        unsafe {
            rb.is_valid(); // will panic if it must
        }
        rb.delete(50);
        assert_eq!(rb.search(50), None);

        rb.delete(500);
        unsafe {
            rb.is_valid(); // will panic if it must
        }
        rb.delete(500);
        assert_eq!(rb.search(500), None);

        rb.delete(5000);
        unsafe {
            rb.is_valid(); // will panic if it must
        }
        rb.delete(5000);
        assert_eq!(rb.search(5000), None);

        rb.delete(50000);
        unsafe {
            rb.is_valid(); // will panic if it must
        }
        rb.delete(50000);
        assert_eq!(rb.search(50000), None);

        rb.delete(500000);
        unsafe {
            rb.is_valid(); // will panic if it must
        }
        rb.delete(500000);
        assert_eq!(rb.search(500000), None);
    }
}
