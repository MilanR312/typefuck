use numbers::NextNumber;

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
macro_rules! ll {
    () => {
        crate::linked_list::End
    };
    ($val:ident) => {
        crate::linked_list::Node<$val, ll!()>
    };
    ($val:ident, $($rest:ident),+) => {
        crate::linked_list::Node<$val, ll!{$($rest),+}>
    };
}
macro_rules! list {
    () => {
        list!(crate::numbers::Zero; )
    };
    ($val:ident) => {
        list!(crate::numbers::Zero; $val)
    };
    ($val:ident, $($rest:ident),+) => {
        list!(crate::numbers::Zero; $val, $($rest),+)
    };
    ($index:ty; $($data:ident),*) => {
        crate::indexed::Indexed<$index, ll!($($data),*)>
    };
}

macro_rules! bf {
    ($ram:ty) => {
        $ram
    };
    ($ram:ty;) => {
        $ram
    };
    ($ram:ty; > $($rest:tt)*) => {
        bf!(MoveRight<$ram>; $($rest)*)
    };
    ($ram:ty; < $($rest:tt)*) => {
        bf!(MoveLeft<$ram>; $($rest)*)
    };
    ($ram:ty; + $($rest:tt)*) => {
        bf!(Incr<$ram>; $($rest)*)
    };
    ($ram:ty; - $($rest:tt)*) => {
        bf!(Decr<$ram>; $($rest)*)
    };
    ($ram:ty; [ $($body:tt)* ] $($rest:tt)*) => {
        bf!(LoopEnd<bf!(LoopStart<$ram>; $($body)* )>; $($rest)*)
    };
    ($ram:ty; . $($rest:tt)*) => {
        bf!(Print<$ram>; $($rest)*)
    };
    ($ram:ty; .. $($rest:tt)*) => {
        bf!($ram; . . $($rest)*)
    };
    ($ram:ty; >> $($rest:tt)*) => {
        bf!($ram; > > $($rest)*)
    };
    ($ram:ty; << $($rest:tt)*) => {
        bf!($ram; < < $($rest)*)
    };
}

trait TypeNamed {
    fn name() -> String;
}
impl TypeNamed for operators::Add {
    fn name() -> String {
        "Add".to_owned()
    }
}
impl TypeNamed for operators::Sub {
    fn name() -> String {
        "Sub".to_owned()
    }
}
impl TypeNamed for numbers::Zero {
    fn name() -> String {
        "0".to_owned()
    }
}
impl<N: numbers::Number> TypeNamed for numbers::NextNumber<N> {
    fn name() -> String {
        format!("{}", 1 + N::eval())
    }
}
impl TypeNamed for linked_list::End {
    fn name() -> String {
        "]".to_owned()
    }
}
impl<Val: TypeNamed, Next: TypeNamed> TypeNamed for linked_list::Node<Val, Next> {
    fn name() -> String {
        format!("{}, {}", Val::name(), Next::name())
    }
}
impl<Idx: TypeNamed, List: TypeNamed> TypeNamed for indexed::Indexed<Idx, List> {
    fn name() -> String {
        format!("List<{},[{}>", Idx::name(), List::name())
    }
}
impl<T: TypeNamed> TypeNamed for instructions::Decr<T> {
    fn name() -> String {
        format!("Decr<{}>", T::name())
    }
}
impl<T: TypeNamed> TypeNamed for instructions::Incr<T> {
    fn name() -> String {
        format!("Incr<{}>", T::name())
    }
}
impl<T: TypeNamed> TypeNamed for instructions::MoveRight<T>{
    fn name() -> String {
        format!("MoveR<{}>", T::name())
    }
}
impl<T: TypeNamed> TypeNamed for instructions::MoveLeft<T>{
    fn name() -> String {
        format!("MoveL<{}>", T::name())
    }
}
impl<T: TypeNamed> TypeNamed for instructions::LoopEnd<T> {
    fn name() -> String {
        format!("LoopEnd<{}>", T::name())
    }
}
impl<T: TypeNamed> TypeNamed for instructions::LoopStart<T> {
    fn name() -> String {
        format!("LoopStart<{}>", T::name())
    }
}
impl<A: TypeNamed, B: TypeNamed> TypeNamed for brainfuck::Interpreter<A,B>{
    fn name() -> String {
        format!("Interpreter<{}, {}>", A::name(), B::name())
    }
}

mod operators {
    pub struct Add;
    pub struct Sub;
}

mod numbers {
    use crate::operators;

    pub struct Zero;
    pub struct NextNumber<N>(N);

    macro_rules! define_numbers {
        ($($number:ident),+) => {
            define_numbers!{@inner Zero, $($number),+}
        };
        (@inner $prev:ident, $next:ident, $($remainder:ident),+) => {
            pub type $next = NextNumber<$prev>;
            define_numbers!{@inner $next, $($remainder),+}
        };
        (@inner $prev:ident, $next: ident) => {
            pub type $next = NextNumber<$prev>;
        };
    }
    // handy aliases
    define_numbers!(One, Two, Three, Four, Five, Six, Seven, Eight, Nine);

    pub trait Operation<Op> {
        type Output;
    }
    impl<N> Operation<operators::Add> for N {
        type Output = NextNumber<N>;
    }
    impl<N> Operation<operators::Sub> for NextNumber<N> {
        type Output = N;
    }
    impl Operation<operators::Sub> for Zero {
        type Output = Zero;
    }

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
    #[cfg(test)]
    mod tests {
        use crate::{
            numbers::{self, Number, Zero},
            operators,
        };
        type Add<T> = <T as numbers::Operation<operators::Add>>::Output;
        type Sub<T> = <T as numbers::Operation<operators::Sub>>::Output;
        #[test]
        fn adds() {
            assert_eq!(<actions!(Zero > Add > Add > Add)>::eval(), 3);
        }
        #[test]
        fn add_sub() {
            assert_eq!(<actions!(Zero > Add > Add > Sub)>::eval(), 1);
        }
    }
}

mod linked_list {
    use crate::numbers::{self, NextNumber, Number, One, Zero};

    pub struct End;
    pub struct Node<Val, Next>(Val, Next);

    pub trait Index<Idx> {
        type Val;
        type List;
    }
    impl<Val, Next> Index<Zero> for Node<Val, Next> {
        type List = Node<Val, Next>;
        type Val = Val;
    }
    impl<Val, Next, Idx> Index<NextNumber<Idx>> for Node<Val, Next>
    where
        Next: Index<Idx>,
    {
        type List = Node<Val, Next>;
        type Val = <Next as Index<Idx>>::Val;
    }
    // when indexing out of bounds grow the ll
    impl Index<Zero> for End {
        type List = Node<Self::Val, End>;
        type Val = Zero;
    }
    impl<Idx> Index<NextNumber<Idx>> for End
    where
        End: Index<Idx>,
    {
        type List = Node<Zero, End>;
        type Val = <End as Index<Idx>>::Val;
    }

    pub trait Len {
        type LenOut;
    }
    impl Len for End {
        type LenOut = Zero;
    }
    impl<V, Next> Len for Node<V, Next>
    where
        Next: Len,
    {
        type LenOut = NextNumber<<Next as Len>::LenOut>;
    }
    pub trait LLOp<Op, Idx> {
        type List;
    }
    impl<Val, Next, Op> LLOp<Op, Zero> for Node<Val, Next>
    where
        Val: numbers::Operation<Op>,
    {
        type List = Node<<Val as numbers::Operation<Op>>::Output, Next>;
    }
    impl<Op> LLOp<Op, Zero> for End
    where
        Zero: numbers::Operation<Op>,
    {
        type List = Node<<Zero as numbers::Operation<Op>>::Output, End>;
    }
    impl<Val, Next, Idx, Op> LLOp<Op, NextNumber<Idx>> for Node<Val, Next>
    where
        Next: LLOp<Op, Idx>,
    {
        type List = Node<Val, <Next as LLOp<Op, Idx>>::List>;
    }
    impl<Idx, Op> LLOp<Op, NextNumber<Idx>> for End
    where
        End: LLOp<Op, Idx>,
    {
        type List = Node<Zero, <End as LLOp<Op, Idx>>::List>;
    }

    pub trait Push<Val> {
        type List;
    }
    impl<ToAdd> Push<ToAdd> for End {
        type List = Node<ToAdd, End>;
    }
    impl<ToAdd, Next, Val> Push<ToAdd> for Node<Val, Next>
    where
        Next: Push<ToAdd>,
    {
        type List = Node<Val, <Next as Push<ToAdd>>::List>;
    }

    pub trait ToVec {
        fn to_vec() -> Vec<usize>;
    }
    impl ToVec for End {
        fn to_vec() -> Vec<usize> {
            vec![]
        }
    }
    impl<N: Number, Next: ToVec> ToVec for Node<N, Next> {
        fn to_vec() -> Vec<usize> {
            let mut a = vec![N::eval()];
            a.extend(Next::to_vec());
            a
        }
    }

    #[cfg(test)]
    mod tests {
        use std::any::type_name;

        use super::ToVec;
        use crate::numbers::{Number, One, Three, Two, Zero};
        type Index<LL, Idx> = <LL as crate::linked_list::Index<Idx>>::Val;
        type Len<LL> = <LL as crate::linked_list::Len>::LenOut;
        type Push<LL, Item> = <LL as crate::linked_list::Push<Item>>::List;
        #[test]
        fn index() {
            type Data = ll!(Zero, One, Two, Three);
            assert_eq!(<Index<Data, Zero>>::eval(), 0);
            assert_eq!(<Index<Data, One>>::eval(), 1);
            assert_eq!(<Index<Data, Two>>::eval(), 2);
            assert_eq!(<Len<Data>>::eval(), 4);
        }
        #[test]
        fn index_out_of_bounds() {
            type Data = ll!();
            println!("{}", type_name::<Data>());
            assert_eq!(<Index<Data, Two>>::eval(), 0);
            // this does not yet grow the array but Index returns the new array that has grown
            assert_eq!(<Len<Data>>::eval(), 0);
        }
        #[test]
        fn test_push() {
            type Data = ll!();
            type Data2 = Push<Push<Data, Two>, One>;
            assert_eq!(Data2::to_vec(), [2, 1]);
        }
    }
}

mod indexed {
    use crate::numbers::Number;
    use crate::{brainfuck, linked_list, numbers};
    use crate::{linked_list::End, numbers::Zero};

    pub struct Indexed<Idx, FirstNode>(Idx, FirstNode);
    pub type EmptyIndexed = Indexed<Zero, End>;

    pub trait Get {
        type Val;
    }
    impl<Idx, FirstNode> Get for Indexed<Idx, FirstNode>
    where
        FirstNode: linked_list::Index<Idx>,
    {
        type Val = <FirstNode as linked_list::Index<Idx>>::Val;
    }
    impl<Ram: Get, Output> Get for brainfuck::Interpreter<Ram, Output>{
        type Val = <Ram as Get>::Val;
    }

    pub trait Len {
        type LenOut;
    }
    impl<Idx, FirstNode> Len for Indexed<Idx, FirstNode>
    where
        FirstNode: linked_list::Len,
    {
        type LenOut = <FirstNode as linked_list::Len>::LenOut;
    }

    pub trait VecOp<Op> {
        type Indexed;
    }
    impl<Op, Idx, FirstNode> VecOp<Op> for Indexed<Idx, FirstNode>
    where
        FirstNode: linked_list::LLOp<Op, Idx>,
    {
        type Indexed = Indexed<Idx, <FirstNode as linked_list::LLOp<Op, Idx>>::List>;
    }

    pub trait IndexOp<Op> {
        type Indexed;
    }
    impl<Op, Idx, FirstNode> IndexOp<Op> for Indexed<Idx, FirstNode>
    where
        Idx: numbers::Operation<Op>,
    {
        type Indexed = Indexed<<Idx as numbers::Operation<Op>>::Output, FirstNode>;
    }

    pub trait Push<Value> {
        type Indexed;
    }
    impl<Value, Idx, FirstNode> Push<Value> for Indexed<Idx, FirstNode>
    where
        FirstNode: linked_list::Push<Value>,
    {
        type Indexed = Indexed<Idx, <FirstNode as linked_list::Push<Value>>::List>;
    }

    pub trait Debug {
        fn index() -> usize;
        fn data() -> Vec<usize>;
    }
    impl<FirstNode, Idx> Debug for Indexed<Idx, FirstNode>
    where
        FirstNode: linked_list::ToVec,
        Idx: Number,
    {
        fn index() -> usize {
            Idx::eval()
        }
        fn data() -> Vec<usize> {
            FirstNode::to_vec()
        }
    }
}

mod instructions {
    use crate::{
        brainfuck, indexed::{self, Get}, numbers::{NextNumber, Zero}, operators
    };

    pub trait Instruction {
        /// execute the instruction and get the result
        type Exec;
        /// create a new instruction from data T
        type Create<E>;
    }
    /*impl<Idx, Val> Instruction for indexed::Indexed<Idx, Val> {
        type Exec = Self;
        type Create<E> = E;
    }*/
    impl<Ram, Output> Instruction for brainfuck::Interpreter<Ram, Output>{
        type Exec = Self;
        type Create<E> = E;
    }
    impl<Index, Val> Instruction for indexed::Indexed<Index, Val>{
        type Create<E> = E;
        type Exec = Self;
    }

    pub struct Decr<T>(T);
    impl<T: Instruction> Instruction for Decr<T>
    where
        <T as Instruction>::Exec: indexed::VecOp<operators::Sub>,
    {
        type Exec = <<T as Instruction>::Exec as indexed::VecOp<operators::Sub>>::Indexed;
        type Create<E> = Decr<<T as Instruction>::Create<E>>;
    }
    pub struct Incr<T>(T);
    impl<T: Instruction> Instruction for Incr<T>
    where
        <T as Instruction>::Exec: indexed::VecOp<operators::Add>,
    {
        type Exec = <<T as Instruction>::Exec as indexed::VecOp<operators::Add>>::Indexed;
        type Create<E> = Incr<<T as Instruction>::Create<E>>;
    }
    pub struct MoveRight<T>(T);
    impl<T: Instruction> Instruction for MoveRight<T>
    where
        <T as Instruction>::Exec: indexed::IndexOp<operators::Add>,
    {
        type Exec = <<T as Instruction>::Exec as indexed::IndexOp<operators::Add>>::Indexed;
        type Create<E> = MoveRight<<T as Instruction>::Create<E>>;
    }

    pub struct MoveLeft<T>(T);
    impl<T: Instruction> Instruction for MoveLeft<T>
    where
        <T as Instruction>::Exec: indexed::IndexOp<operators::Sub>
    {
        type Create<E> = MoveLeft<E>;
        type Exec = <<T as Instruction>::Exec as indexed::IndexOp<operators::Sub>>::Indexed;
    }

    pub struct LoopStart<T>(T);
    impl<T: Instruction> Instruction for LoopStart<T> {
        type Exec = <T as Instruction>::Exec;
        type Create<E> = LoopStart<<T as Instruction>::Create<T>>;
    }
    pub struct LoopEnd<T>(T);

    pub trait Loop<Cond> {
        type LoopOut;
    }
    impl<T: Instruction> Loop<Zero> for LoopEnd<T> {
        type LoopOut = <T as Instruction>::Exec;
    }

    impl<T: Instruction, Val> Loop<NextNumber<Val>> for LoopEnd<T> {
        type LoopOut = LoopEnd<<T as Instruction>::Create<<T as Instruction>::Exec>>;
    }

    impl<T: Instruction> Instruction for LoopEnd<T>
    where
        Self: Loop<GetCondition<Self>> + Get,
        <Self as Loop<GetCondition<Self>>>::LoopOut: Instruction,
    {
        type Exec = <<Self as Loop<GetCondition<Self>>>::LoopOut as Instruction>::Exec;
        type Create<E> = E;
    }
    pub struct Print<T>(T);
    impl<T: Instruction> Instruction for Print<T>
    where <T as Instruction>::Exec: brainfuck::Print
    {
        type Exec = <
            <T as Instruction>::Exec as brainfuck::Print
        >::Out;
        type Create<E> = Print<E>;
    }
    /*impl<T: LoopInstruction> LoopInstruction for LoopEnd<T> {
        type Exec = <T as LoopInstruction>::Exec;
        type Create<E> = LoopEnd<<T as LoopInstruction>::Create<E>>;
    }*/

    type GetCondition<T> = <T as indexed::Get>::Val;
    macro_rules! getter {
        ($($name:ident),+) => {
            $(impl<T: indexed::Get> indexed::Get for $name<T> {
                type Val = <T as indexed::Get>::Val;
            })*
        };
    }
    getter!(LoopStart, LoopEnd, Decr, MoveRight, MoveLeft, Incr);

    type Execute<T> = <T as Instruction>::Exec;
    type GetRam<T> = <T as brainfuck::Debug>::Ram;
    

    #[cfg(test)]
    mod tests {
        use std::any::type_name;

        use crate::{
            brainfuck::{self, Interpreter}, indexed::{Debug, Indexed}, instructions::{
                Decr, Execute, GetCondition, GetRam, Incr, Instruction, Loop, LoopEnd, LoopStart, MoveRight, MoveLeft, Print
            }, linked_list::{End, Node}, numbers::{One, Three, Two, Zero}, TypeNamed
        };

        #[test]
        fn test_loop() {
            // memory is init to [3] with pointer at index 0
            type Ram = list![Three];
            assert_eq!(Ram::data(), [3]);
            assert_eq!(Ram::index(), 0);
            type Ram2 = Interpreter<Ram, list![]>;
            type Code = bf!(Ram2; [-]>+.);
            //type Code = bf!(Ram; [-]>+);
            // evaluate the code
            type Output = GetRam<Execute<Code>>;
            assert_eq!(Output::data(), [0, 1]);
            assert_eq!(Output::index(), 1);
        }
        #[test]
        fn test_loop2(){
            type Ram = Interpreter<list![], list![]>;
            type Code = bf!(Ram; ++[-]);
            type Output = GetRam<Execute<Code>>;
            assert_eq!(Output::data(), [0]);
        }
    }
}

mod brainfuck {
    use crate::indexed;

    pub struct Interpreter<Ram, Output>(Ram, Output);
    pub trait Debug {
        type Output;
        type Ram;
    }
    impl<Ram, Output> Debug for Interpreter<Ram, Output>{
        type Output = Output;
        type Ram = Ram;
    }

    impl<Op, Ram, Output> indexed::VecOp<Op> for Interpreter<Ram, Output>
    where Ram: indexed::VecOp<Op>
    {
        type Indexed = Interpreter<<Ram as indexed::VecOp<Op>>::Indexed, Output>;
    }
    impl<Op, Ram, Output> indexed::IndexOp<Op> for Interpreter<Ram, Output>
    where Ram: indexed::IndexOp<Op>
    {
        type Indexed = Interpreter<<Ram as indexed::IndexOp<Op>>::Indexed, Output>;
    }

    pub trait Print {
        type Out;
    }
    impl<Ram, Output> Print for Interpreter<Ram, Output>
    where Ram: indexed::Get,
        Output: indexed::Push< <Ram as indexed::Get>::Val >
     {
        type Out = Interpreter<
            Ram,
            < Output as indexed::Push<
                <Ram as indexed::Get>::Val
                >
            >::Indexed
        >;
    }
}

fn main() {
    type Init = crate::brainfuck::Interpreter<list![], list![]>;
    use crate::instructions::*;
    type Code = bf!(Init; ++++++++++[-]);
  //type Code = bf!(Init; ++++++++++[>+++++++>++++++++++>+++>+<<<<-]);
  //type Code = bf!(Init; ++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.);
    type Output = <Code as Instruction>::Exec;
    type String = <Output as crate::brainfuck::Debug>::Output;
    type Ram = <Output as crate::brainfuck::Debug>::Ram;
    println!("{}", Ram::name());
    println!("{}", String::name());
}
