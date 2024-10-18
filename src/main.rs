#![recursion_limit = "256"]
use brainfuck::GetOutput;
use indexed::Debug;
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
    ($ram:ty; <- $($rest:tt)*) => {
        bf!($ram; < - $($rest)*)
    };
    ($ram:ty; -> $($rest:tt)*) => {
        bf!($ram; - > $($rest)*)
    }
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
impl<T: TypeNamed> TypeNamed for instructions::MoveRight<T> {
    fn name() -> String {
        format!("MoveR<{}>", T::name())
    }
}
impl<T: TypeNamed> TypeNamed for instructions::MoveLeft<T> {
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
impl<A: TypeNamed, B: TypeNamed> TypeNamed for brainfuck::InterpreterBase<A, B> {
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
            // whilst our numbers can overflow a usize, rustc should reach the recursion limit/crash/oom before this happens
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
    #[derive(Debug)]
    pub struct InvalidCharError;
    pub trait ToVec {
        fn to_vec() -> Vec<usize>;
        fn to_string() -> Result<String, InvalidCharError> {
            let data = Self::to_vec()
                .into_iter()
                .map(|x| u8::try_from(x).map_err(|_| InvalidCharError))
                .collect::<Result<Vec<_>, _>>()?;
            String::from_utf8(data).map_err(|_| InvalidCharError)
        }
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
    use crate::linked_list::InvalidCharError;
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
    impl<Ram: Get, Output> Get for brainfuck::InterpreterBase<Ram, Output> {
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
        fn string() -> Result<String, InvalidCharError>;
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
        fn string() -> Result<String, InvalidCharError> {
            FirstNode::to_string()
        }
    }
}

mod instructions {
    use crate::{
        brainfuck,
        indexed::{self, Get},
        numbers::{NextNumber, Zero},
        operators,
    };

    pub trait Instruction {
        /// execute the instruction and get the result
        type Exec;
        /// create a new instruction from data T
        type Create<E>;
        type Interpreter;
    }
    /*impl<Idx, Val> Instruction for indexed::Indexed<Idx, Val> {
        type Exec = Self;
        type Create<E> = E;
    }*/
    impl<Ram, Output> Instruction for brainfuck::InterpreterBase<Ram, Output> {
        type Exec = Self;
        type Create<E> = E;
        type Interpreter = Self;
    }
    impl<Index, Val> Instruction for indexed::Indexed<Index, Val> {
        type Create<E> = E;
        type Exec = Self;
        type Interpreter = Self;
    }

    pub struct Decr<T>(T);
    impl<T: Instruction> Instruction for Decr<T>
    where
        <T as Instruction>::Exec: indexed::VecOp<operators::Sub>,
    {
        type Exec = <<T as Instruction>::Exec as indexed::VecOp<operators::Sub>>::Indexed;
        type Create<E> = Decr<<T as Instruction>::Create<E>>;
        type Interpreter = <T as Instruction>::Interpreter;
    }
    pub struct Incr<T>(T);
    impl<T: Instruction> Instruction for Incr<T>
    where
        <T as Instruction>::Exec: indexed::VecOp<operators::Add>,
    {
        type Exec = <<T as Instruction>::Exec as indexed::VecOp<operators::Add>>::Indexed;
        type Create<E> = Incr<<T as Instruction>::Create<E>>;
        type Interpreter = <T as Instruction>::Interpreter;
    }
    pub struct MoveRight<T>(T);
    impl<T: Instruction> Instruction for MoveRight<T>
    where
        <T as Instruction>::Exec: indexed::IndexOp<operators::Add>,
    {
        type Exec = <<T as Instruction>::Exec as indexed::IndexOp<operators::Add>>::Indexed;
        type Create<E> = MoveRight<<T as Instruction>::Create<E>>;
        type Interpreter = <T as Instruction>::Interpreter;
    }

    pub struct MoveLeft<T>(T);
    impl<T: Instruction> Instruction for MoveLeft<T>
    where
        <T as Instruction>::Exec: indexed::IndexOp<operators::Sub>,
    {
        type Exec = <<T as Instruction>::Exec as indexed::IndexOp<operators::Sub>>::Indexed;
        type Create<E> = MoveLeft<<T as Instruction>::Create<E>>;
        type Interpreter = <T as Instruction>::Interpreter;
    }

    pub struct LoopStart<T>(T);
    impl<T: Instruction> Instruction for LoopStart<T> {
        type Exec = <T as Instruction>::Exec;
        type Create<E> = LoopStart<E>;
        type Interpreter = <T as Instruction>::Interpreter;
    }
    pub struct LoopEnd<T>(T);

    pub trait Loop<Cond> {
        type LoopOut;
    }
    impl<T: Instruction> Loop<Zero> for LoopEnd<T> {
        type LoopOut = <T as Instruction>::Interpreter;
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
        type Interpreter = <T as Instruction>::Interpreter;
    }

    pub struct Print<T>(T);
    impl<T: Instruction> Instruction for Print<T>
    where
        <T as Instruction>::Exec: brainfuck::Print,
    {
        type Exec = <<T as Instruction>::Exec as brainfuck::Print>::Out;
        type Create<E> = Print<E>;
        type Interpreter = <T as Instruction>::Interpreter;
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
    getter!(LoopEnd, Decr, MoveRight, MoveLeft, Incr);
    impl<T> Get for LoopStart<T>
    where
        Self: Instruction,
        <Self as Instruction>::Exec: Get,
    {
        type Val = <<Self as Instruction>::Exec as Get>::Val;
    }

    pub type Execute<T> = <T as Instruction>::Exec;
    type GetRam<T> = <T as brainfuck::Debug>::Ram;

    #[cfg(test)]
    mod tests {
        use std::any::type_name;

        use crate::{
            brainfuck::{self, InterpreterBase},
            indexed::{self, Debug, Get, Indexed},
            instructions::{
                Decr, Execute, GetCondition, GetRam, Incr, Instruction, Loop, LoopEnd, LoopStart,
                MoveLeft, MoveRight, Print,
            },
            linked_list::{End, Node},
            numbers::{Nine, Number, One, Three, Two, Zero},
            TypeNamed,
        };

        #[test]
        fn test_loop() {
            // memory is init to [3] with pointer at index 0
            type Ram = list![Three];
            assert_eq!(Ram::data(), [3]);
            assert_eq!(Ram::index(), 0);
            type Ram2 = InterpreterBase<Ram, list![]>;
            type Code = bf!(Ram2; [-]>+.);
            //type Code = bf!(Ram; [-]>+);
            // evaluate the code
            type Output = GetRam<Execute<Code>>;
            assert_eq!(Output::data(), [0, 1]);
            assert_eq!(Output::index(), 1);
        }
        #[test]
        fn test_loop2() {
            type Ram = InterpreterBase<list![], list![]>;
            type Code = bf!(Ram; ++[-]+);
            type Output = GetRam<Execute<Code>>;
            //assert_eq!(<Output as indexed::Get>::Val::eval(), 0);
            assert_eq!(Output::data(), [1]);
        }
        #[test]
        fn empty_loop() {
            type Ram = InterpreterBase<list![Zero, One], list![]>;
            type Code = bf!(Ram; [>-<]);
            type Output = GetRam<Execute<Code>>;
            assert_eq!(Output::data(), [0, 1]);
        }
        #[test]
        fn base_loop_3() {
            type Ram = InterpreterBase<list![Nine], list![]>;
            type Code = bf!(Ram; [>+<-]);

            type Iter1 = <Code as Loop<GetCondition<Code>>>::LoopOut;
            assert_eq!(<Iter1 as brainfuck::Debug>::Ram::data(), [8, 1]);

            type Iter2 = <Iter1 as Loop<GetCondition<Iter1>>>::LoopOut;
            assert_eq!(<Iter2 as brainfuck::Debug>::Ram::data(), [7, 2]);

            type Iter3 = <Iter2 as Loop<GetCondition<Iter2>>>::LoopOut;
            assert_eq!(<Iter3 as brainfuck::Debug>::Ram::data(), [6, 3]);

            type Iter4 = <Iter3 as Loop<GetCondition<Iter3>>>::LoopOut;
            assert_eq!(<Iter4 as brainfuck::Debug>::Ram::data(), [5, 4]);

            type Iter5 = <Iter4 as Loop<GetCondition<Iter4>>>::LoopOut;
            assert_eq!(<Iter5 as brainfuck::Debug>::Ram::data(), [4, 5]);

            type Iter6 = <Iter5 as Loop<GetCondition<Iter5>>>::LoopOut;
            assert_eq!(<Iter6 as brainfuck::Debug>::Ram::data(), [3, 6]);

            type Iter7 = <Iter6 as Loop<GetCondition<Iter6>>>::LoopOut;
            assert_eq!(<Iter7 as brainfuck::Debug>::Ram::data(), [2, 7]);

            type Iter8 = <Iter7 as Loop<GetCondition<Iter7>>>::LoopOut;
            assert_eq!(<Iter8 as brainfuck::Debug>::Ram::data(), [1, 8]);

            type Iter9 = <Iter8 as Loop<GetCondition<Iter8>>>::LoopOut;
            assert_eq!(<Iter9 as brainfuck::Debug>::Ram::data(), [0, 9]);

            type Iter10 = <Iter9 as Loop<GetCondition<Iter9>>>::LoopOut;
            assert_eq!(<Iter10 as brainfuck::Debug>::Ram::data(), [0, 9]);
            //assert_eq!(Output::data(), [0,2]);
        }

        #[test]
        fn test_loop_3() {
            type Ram = InterpreterBase<list![Nine], list![]>;
            type Code = bf!(Ram; [>+<-]);
            type Output = Execute<Code>;
            //panic!("{}", Output::name());
            assert_eq!(<Output as brainfuck::Debug>::Ram::data(), [0, 9]);
        }
    }
}

mod brainfuck {
    use crate::{indexed, instructions};

    pub struct InterpreterBase<Ram, Output>(Ram, Output);
    pub trait Debug {
        type Output;
        type Ram;
    }
    impl<Ram, Output> Debug for InterpreterBase<Ram, Output> {
        type Output = Output;
        type Ram = Ram;
    }
    macro_rules! debug_wrapper {
        ($($items:ident),*) => {
                $(
                    impl<T: Debug> Debug for instructions::$items<T>{
                        type Output = <T as Debug>::Output;
                        type Ram = <T as Debug>::Ram;
                    }
                )*
        };
    }
    debug_wrapper!(Incr, Decr, MoveRight, MoveLeft, LoopStart, LoopEnd);

    impl<Op, Ram, Output> indexed::VecOp<Op> for InterpreterBase<Ram, Output>
    where
        Ram: indexed::VecOp<Op>,
    {
        type Indexed = InterpreterBase<<Ram as indexed::VecOp<Op>>::Indexed, Output>;
    }
    impl<Op, Ram, Output> indexed::IndexOp<Op> for InterpreterBase<Ram, Output>
    where
        Ram: indexed::IndexOp<Op>,
    {
        type Indexed = InterpreterBase<<Ram as indexed::IndexOp<Op>>::Indexed, Output>;
    }

    pub trait Print {
        type Out;
    }
    impl<Ram, Output> Print for InterpreterBase<Ram, Output>
    where
        Ram: indexed::Get,
        Output: indexed::Push<<Ram as indexed::Get>::Val>,
    {
        type Out = InterpreterBase<Ram, <Output as indexed::Push<<Ram as indexed::Get>::Val>>::Indexed>;
    }
    pub type GetOutput<T> = <T as Debug>::Output;
    pub type Interpreter = InterpreterBase<list!(), list!()>;
}



fn main() {
    use crate::instructions::*;
    type Base = crate::brainfuck::Interpreter;
    type Code = bf!(Base; +++++++++[>++++++++>+++++++++++>++++>+++++++++>+++++++++++++<<<<<-]>.>++.+++++++..+++.>----.>+++.>++++.<<<+.-----------.>>>------.<<+.);
    type Final = Execute<Code>;
    type Output = GetOutput<Final>;

    // unwrap since a program output may not be a valid string
    let data = Output::string().unwrap();
    assert_eq!(data, "Hello Types!");
}
