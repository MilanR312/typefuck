use std::{any::type_name, marker::PhantomData};

macro_rules! actions {
    ($base:ty) => {
        $base
    };
    ($base:ty>$($rest:tt)*) => {
        actions!{ @actions $base, $($rest)*}
    };
    (@actions $prev:ty, $operation:ident) => {
        $operation<$prev>
    };
    (@actions $prev:ty, $operation:ident>$($rest:tt)*) => {
        actions!(@actions $operation<$prev>, $($rest)*)
    }
}

mod operators {
    pub struct Add;
    pub struct Sub;
}
mod numbers {
    use crate::operators;

    pub trait Number {
        fn eval() -> usize;
    }
    impl Number for Zero {
        fn eval() -> usize {
            0
        }
    }
    impl<N: Number> Number for NextNumber<N> {
        fn eval() -> usize {
            1 + N::eval()
        }
    }
    pub struct Zero;
    pub struct NextNumber<N>(N);

    pub trait NumOperator<Operator> {
        type Output;
    }
    impl NumOperator<operators::Add> for Zero {
        type Output = NextNumber<Zero>;
    }
    // no zero impl

    impl<N> NumOperator<operators::Add> for NextNumber<N> {
        type Output = NextNumber<NextNumber<N>>;
    }
    impl<N> NumOperator<operators::Sub> for NextNumber<N> {
        type Output = N;
    }

    pub type Add<N> = <N as NumOperator<operators::Add>>::Output;
    pub type Sub<N> = <N as NumOperator<operators::Sub>>::Output;

    // some handy aliases
    pub type One = NextNumber<Zero>;
    pub type Two = NextNumber<One>;
    pub type Three = NextNumber<Two>;
    pub type Four = NextNumber<Three>;
    pub type Five = NextNumber<Four>;

    #[cfg(test)]
    mod tests {
        use crate::{Add, Number, Zero};

        #[test]
        fn add_test() {
            type A = Zero;
            type B = Add<Add<Add<A>>>;
            assert_eq!(B::eval(), 3);
        }
    }
}
use brainfuck::{Incr, Interpreter, MoveRight};
use numbers::*;

mod list {
    use crate::{operators, Add, NextNumber, NumOperator, Number, Sub, Zero};

    pub trait ListEll {
        fn get() -> Vec<usize>;
    }
    impl ListEll for End {
        fn get() -> Vec<usize> {
            vec![]
        }
    }
    impl<V: Number, Next: ListEll> ListEll for Node<V, Next> {
        fn get() -> Vec<usize> {
            let mut out = vec![V::eval()];
            out.extend(Next::get());
            out
        }
    }
    pub struct End;
    /// a type level linked list, can be randomly indexed
    pub struct Node<Val, Next>(Val, Next);

    pub trait NodeOperator<Operator, Index> {
        type Output;
    }
    // base implement for index zero, this means we reached the data
    impl<Op, Val: NumOperator<Op>, Next> NodeOperator<Op, Zero> for Node<Val, Next> {
        type Output = Node<<Val as NumOperator<Op>>::Output, Next>;
    }
    // recursive impl
    impl<Op, Val, Next: NodeOperator<Op, Index>, Index> NodeOperator<Op, NextNumber<Index>>
        for Node<Val, Next>
    {
        type Output = Node<Val, <Next as NodeOperator<Op, Index>>::Output>;
    }
    // to make the list growable, if we reach the end node, we generate a new node
    impl NodeOperator<operators::Add, Zero> for End {
        type Output = Node<<Zero as NumOperator<operators::Add>>::Output, End>;
    }
    impl<Idx> NodeOperator<operators::Add, NextNumber<Idx>> for End
    where
        End: NodeOperator<operators::Add, Idx>,
    {
        type Output = Node<Zero, <End as NodeOperator<operators::Add, Idx>>::Output>;
    }

    pub type NodeAdd<List, Idx> = <List as NodeOperator<operators::Add, Idx>>::Output;
    pub type NodeSub<List, Idx> = <List as NodeOperator<operators::Sub, Idx>>::Output;

    pub trait ReadNode<Idx> {
        type Value;
        type List;
    }
    impl<Val, Next> ReadNode<Zero> for Node<Val, Next> {
        type List = Node<Val, Next>;
        type Value = Val;
    }
    impl<Val, Next, Index> ReadNode<NextNumber<Index>> for Node<Val, Next>
    where
        Next: ReadNode<Index>,
    {
        type List = Node<Val, Next>;
        type Value = <Next as ReadNode<Index>>::Value;
    }

    #[cfg(test)]
    mod tests {
        use std::any::type_name;

        use crate::{End, ListEll, Node, NodeAdd, Two, Zero};

        #[test]
        fn grow_nodes() {
            type Empty = End;
            type SingleEll = NodeAdd<Empty, Zero>;
            assert_eq!(SingleEll::get(), [1]);
            type TripleEll = NodeAdd<SingleEll, Two>; // x[2] += 1
            assert_eq!(TripleEll::get(), [1, 0, 1]);
        }
    }
}
use list::*;

mod brainfuck {
    use crate::{operators, ListEll, NodeOperator, NumOperator, Number};

    pub struct Interpreter<Node, Idx>(Node, Idx);

    pub trait IndexOperator<Operand> {
        type Output;
    }
    impl<Op, Node, Idx: NumOperator<Op>> IndexOperator<Op> for Interpreter<Node, Idx> {
        type Output = Interpreter<Node, <Idx as NumOperator<Op>>::Output>;
    }
    pub trait ListOperator<Operand> {
        type Output;
    }
    impl<Op, Node: NodeOperator<Op, Idx>, Idx> ListOperator<Op> for Interpreter<Node, Idx> {
        type Output = Interpreter<<Node as NodeOperator<Op, Idx>>::Output, Idx>;
    }

    /*pub trait ReadList {
        type Value;
        type List;
    }
    impl<Node, Idx> ReadList for Interpreter<Node, Idx>
    where
        Node: ReadNode<Idx>,
    {
        type List = List<Node, Idx>;
        type Value = <Node as ReadNode<Idx>>::Value;
    }*/

    pub trait ListInsert {
        type Output;
    }

    impl<Node: ListEll, Idx: Number> Interpreter<Node, Idx> {
        pub fn debug() {
            println!("{{ index={}, data={:?} }}", Idx::eval(), Node::get());
        }
        pub fn data() -> Vec<usize> {
            Node::get()
        }
        pub fn index() -> usize {
            Idx::eval()
        }
    }
    pub type MoveRight<List> = <List as IndexOperator<operators::Add>>::Output;
    pub type MoveLeft<List> = <List as IndexOperator<operators::Sub>>::Output;
    pub type Incr<List> = <List as ListOperator<operators::Add>>::Output;
    pub type Decr<List> = <List as ListOperator<operators::Sub>>::Output;
    #[cfg(test)]
    mod tests {
        use crate::{
            brainfuck::{Incr, Interpreter, MoveRight},
            End, Zero,
        };

        #[test]
        fn manipulate_list() {
            type Base = Interpreter<End, Zero>;
            type Data = actions!(Base > MoveRight > Incr > Incr > MoveRight > Incr);
            assert_eq!(Data::data(), [0, 2, 1]);
            assert_eq!(Data::index(), 2);
        }
    }
    //pub type Get<List> = <List as >::Value;
}

mod loop_instr {
    use crate::{NextNumber, Zero};

    pub trait LoopInner<Condition> {
        type Output;
    }
    pub struct LoopItem<Inner>(Inner);

    impl<Inner, Next> LoopInner<Zero> for LoopItem<Inner, Next> {
        type Output = Next;
    }
    impl<Number, Inner, Next> LoopInner<NextNumber<Number>> for LoopItem<Inner, Next> {
        type Output = LoopItem<Inner, Next>;
    }

    #[cfg(test)]
    mod tests {
        use crate::loop_instr::LoopItem;

        #[test]
        fn loop_basic() {
            type Data = LoopItem<>
        }
    }
}

fn main() {
    //type Ptr = Move<Move<Move<Move<PtrBase, Right>, Right>, Right>, Left>;
    type BrainfuckStack = Interpreter<End, Zero>;
    type Output = actions!(BrainfuckStack > MoveRight > Incr > Incr > MoveRight > Incr);
    assert_eq!(Output::data(), [0, 2, 1]);
}
