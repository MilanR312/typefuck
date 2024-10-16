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

macro_rules! push_front_array {
    ($val:literal, [$($rest:literal),*]) => {
        [$val, $($rest)*,]
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
impl<Data: TypeNamed> TypeNamed for loopinstr::StartLoop<Data> {
    fn name() -> String {
        format!("Start<{}>", Data::name())
    }
}
impl<Data: TypeNamed> TypeNamed for loopinstr::DeferredIncr<Data> {
    fn name() -> String {
        format!("DIncr<{}>", Data::name())
    }
}
impl<Data: TypeNamed> TypeNamed for loopinstr::DeferredMoveRight<Data> {
    fn name() -> String {
        format!("DRight<{}>", Data::name())
    }
}
impl<Data: TypeNamed> TypeNamed for loopinstr::EndLoop<Data> {
    fn name() -> String {
        format!("End<{}>", Data::name())
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
    use crate::{linked_list, numbers};
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
    use crate::{indexed, linked_list, operators};

    pub trait Instruction {}
    impl<Idx, Val, Next> Instruction for indexed::Indexed<Idx, linked_list::Node<Val, Next>> {}

    // data should be `Indexed`
    pub type MoveRight<Data> = <Data as indexed::IndexOp<operators::Add>>::Indexed;
    pub type MoveLeft<Data> = <Data as indexed::IndexOp<operators::Sub>>::Indexed;
    pub type Incr<Data> = <Data as indexed::VecOp<operators::Add>>::Indexed;
    pub type Decr<Data> = <Data as indexed::VecOp<operators::Sub>>::Indexed;

    #[cfg(test)]
    mod tests {
        use crate::{
            indexed::{Debug, EmptyIndexed},
            instructions::{Incr, MoveLeft, MoveRight},
        };

        #[test]
        fn basic_brainfuck() {
            type Ram = EmptyIndexed;
            type Output =
                actions!(Ram > Incr > MoveRight > MoveRight > Incr > Incr > MoveLeft > Incr);
            assert_eq!(Output::data(), [1, 1, 2]);
            assert_eq!(Output::index(), 1);
        }
    }
}

mod loopinstr {
    // represents a while (value != 0) loop

    use std::{any::type_name, marker::PhantomData};

    use crate::{
        indexed::{self, EmptyIndexed, Get, IndexOp, VecOp},
        instructions::{self, Incr, Instruction, MoveRight},
        numbers::{NextNumber, One, Zero},
        operators,
    };

    pub struct StartLoop<Data>(Data);
    //hiiiii milman :3 Voetie was here <33

    impl<Op, Data> VecOp<Op> for StartLoop<Data>
    where
        Data: VecOp<Op>,
    {
        type Indexed = <Data as VecOp<Op>>::Indexed;
    }

    pub struct DeferredIncr<D>(D);
    pub struct DeferredMoveRight<D>(D);

    type GetCondition<T> = <T as indexed::Get>::Val;
    macro_rules! getter {
        ($($name:ident),+) => {
            $(impl<T: indexed::Get> indexed::Get for $name<T> {
                type Val = <T as indexed::Get>::Val;
            })*
        };
    }
    getter!(StartLoop, DeferredIncr, DeferredMoveRight);
    impl<Op, T: indexed::IndexOp<Op>> indexed::IndexOp<Op> for StartLoop<T> {
        type Indexed = StartLoop<<T as indexed::IndexOp<Op>>::Indexed>;
    }
    /*
    trait GetCondition {
        type Cond;
    }

    impl<Cond, D> GetCondition for StartLoop<Cond, D> {
        type Cond = Cond;
    }
    impl<T: GetCondition> GetCondition for DeferredIncr<T> {
        type Cond = <T as GetCondition>::Cond;
    }
    impl<T: GetCondition> GetCondition for DeferredMoveRight<T> {
        type Cond = <T as GetCondition>::Cond;
    }*/

    pub trait ExecLoop<Condition> {
        type Exec;
        type NewLoop;
    }
    // executes for if the condition is zero
    impl<T> ExecLoop<Zero> for StartLoop<T> {
        type Exec = T;
        type NewLoop = T;
    }

    impl<T: ExecLoop<Zero>> ExecLoop<Zero> for DeferredIncr<T> {
        //type Output = <T as ExecLoop<Zero>>::Output;
        type Exec = <T as ExecLoop<Zero>>::Exec;
        type NewLoop = T;
    }
    impl<T: ExecLoop<Zero>> ExecLoop<Zero> for DeferredMoveRight<T> {
        //type Output = <T as ExecLoop<Zero>>::Output;
        type Exec = <T as ExecLoop<Zero>>::Exec;
        type NewLoop = T;
    }

    // executes for if the condition isnt zero
    impl<Num, T> ExecLoop<NextNumber<Num>> for StartLoop<T>
    //where
    //T: ExecLoop<<T as Get>::Val> + Get,
    {
        // edit this !!!!
        //type Output = StartLoop<T>;
        type Exec = T;
        type NewLoop = StartLoop<T>;
    }
    impl<Num, T> ExecLoop<NextNumber<Num>> for DeferredIncr<T>
    where
        T: ExecLoop<NextNumber<Num>>,
        <T as ExecLoop<NextNumber<Num>>>::Exec: VecOp<operators::Add>,
    {
        //type Output = DeferredIncr<Incr<<T as ExecLoop<NextNumber<Num>>>::Output>>;
        type Exec = Incr<<T as ExecLoop<NextNumber<Num>>>::Exec>;
        type NewLoop = DeferredIncr<<T as ExecLoop<NextNumber<Num>>>::NewLoop>;
    }

    impl<Num, T> ExecLoop<NextNumber<Num>> for DeferredMoveRight<T>
    where
        T: ExecLoop<NextNumber<Num>>,
        <T as ExecLoop<NextNumber<Num>>>::Exec: IndexOp<operators::Add>, //<T as ExecLoop<Nex
    {
        //type Output = DeferredMoveRight<MoveRight<<T as ExecLoop<NextNumber<Num>>>::Output>>;
        type Exec = MoveRight<<T as ExecLoop<NextNumber<Num>>>::Exec>;
        type NewLoop = DeferredMoveRight<<T as ExecLoop<NextNumber<Num>>>::NewLoop>;
    }

    pub struct EndLoop<T>(T);
    pub trait GetEnd {
        type Output;
    }
    impl<T> GetEnd for EndLoop<T>
    where
        T: Get,
        EndLoop<T>: ExecLoop<<T as Get>::Val>,
    {
        type Output = <EndLoop<T> as ExecLoop<GetCondition<T>>;
        //type Output = <EndLoop<T> as ExecLoop<GetCondition<T>>>::Exec;
    }
    impl<T> ExecLoop<Zero> for EndLoop<T> {
        type Exec = T;
        type NewLoop = T;
    }
    impl<T, Num> ExecLoop<NextNumber<Num>> for EndLoop<T>
    where
        T: ExecLoop<NextNumber<Num>>,
    {
        //type Output = EndLoop<<T as ExecLoop<NextNumber<Num>>>::Output>;
        type Exec = <T as ExecLoop<NextNumber<Num>>>::Exec;
        type NewLoop = EndLoop<<T as ExecLoop<NextNumber<Num>>>::NewLoop>;
    }
    pub type Eval<T> = <T as GetEnd>::Output;

    #[cfg(test)]
    mod tests {
        use crate::{
            indexed::{Debug, Indexed},
            instructions::MoveRight,
            linked_list::{End, Node},
            loopinstr::{DeferredMoveRight, EndLoop, Eval, GetCondition},
            TypeNamed,
        };
        use std::any::type_name;

        use crate::{
            indexed::EmptyIndexed,
            loopinstr::{DeferredIncr, ExecLoop, StartLoop},
            numbers::{Number, One, Zero},
        };

        #[test]
        fn non_loop() {
            type T = DeferredIncr<DeferredIncr<StartLoop<Indexed<Zero, Node<Zero, End>>>>>;
            assert_eq!(<GetCondition<T>>::eval(), 0);
            //type Out = <T as ExecLoop<GetCondition<T>>>::Output;
            // since the condition is 0, all deferred instructions are canceled and the output is 0
            //assert_eq!(Out::data(), [0]);
        }
        /*#[test]
        fn single_loop() {
            type T = DeferredIncr<DeferredIncr<StartLoop<Zero, EmptyIndexed>>>;
            type Out = <T as ExecLoop<One>>::Output;
            assert_eq!(Out::data(), [2]);
        }*/
        #[test]
        fn multi_loop() {
            type Data = ll!(One, One);
            type Ram = Indexed<Zero, Data>;
            type T = DeferredMoveRight<DeferredIncr<StartLoop<Ram>>>;
            assert_eq!(<GetCondition<T>>::eval(), 1);

            type Output = Eval<EndLoop<T>>;
            //assert_eq!(Out::data(), [1, 1]);
            //assert_eq!(Out::index(), 2);
            panic!("{}", Output::name());
        }
    }

    // will be looped
    /*trait GetShouldRepeat {}
    impl<T: Instruction> GetShouldRepeat for T {}
    impl<D, Cond> GetShouldRepeat for StartLoop<D, Cond> {}

    struct StartLoop<D, Condition>(D, Condition);
    trait LoopTemp {
        type Output;
    }
    impl<Instructions> LoopTemp for Instructions {
        type Output = ();
    }
    fn test() {
        use crate::instructions::*;
        type T = Incr<StartLoop<Out, Zero>>;
        println!("{}", type_name::<T>());

        fn is_instruction<T: Instruction>() {}
        type Ram = EmptyIndexed;
        type Out = actions!(Ram > Incr > MoveRight > Incr > Incr);
        is_instruction::<Out>();
        type Loop = StartLoop<Out, Zero>;
        //is_instruction::<Loop>(); doesnt compile :)
    }*/
    // ++[--]
    // EndLoop<Decr<Decr<StartLoop<Incr<Incr<Ram>>>>>
    // exec -> EndLoop<Decr<Decr<StartLoop<_Ty_>>>
    // Check memory -> if 0
    // exec -> _Ty_
    // if != 0
    // EndLoop<Decr<Decr<StartLoop< _Ty2_ >>> met _Ty2_ = Decr<Decr<_Ty_>>

    // of

    // ++[--]
    // EndLoop<Decr<Decr<StartLoop<Incr<Incr<Ram>>>>>
    // exec -> EndLoop<Decr<Decr<StartLoop< _Ty_ >>>>
    // check memory -> if 0
    // set false in StartLoop
    // in endloop do nothing -> _Ty_
}

mod brainfuck {
    //pub struct Interpreter<Output, Ram, Instructions>(Output, Ram, Instructions);

    //pub trait
}

fn main() {}
