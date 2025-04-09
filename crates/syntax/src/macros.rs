#[macro_export]
macro_rules! T {
    ("eval") => {
        $crate::token::TokenKind::Eval
    };
    ("die") => {
        $crate::token::TokenKind::Die
    };
    ("self") => {
        $crate::token::TokenKind::Self_
    };
    ("parent") => {
        $crate::token::TokenKind::Parent
    };
    ("`") => {
        $crate::token::TokenKind::Backtick
    };
    ("<<<") => {
        $crate::token::TokenKind::DocumentStart(_)
    };
    (">>>") => {
        $crate::token::TokenKind::DocumentEnd
    };
    ("from") => {
        $crate::token::TokenKind::From
    };
    ("print") => {
        $crate::token::TokenKind::Print
    };
    ("$") => {
        $crate::token::TokenKind::Dollar
    };
    ("__halt_compiler") => {
        $crate::token::TokenKind::HaltCompiler
    };
    ("readonly") => {
        $crate::token::TokenKind::Readonly
    };
    ("global") => {
        $crate::token::TokenKind::Global
    };
    ("abstract") => {
        $crate::token::TokenKind::Abstract
    };
    ("&") => {
        $crate::token::TokenKind::Ampersand
    };
    ("&=") => {
        $crate::token::TokenKind::AmpersandEqual
    };
    ("&&") => {
        $crate::token::TokenKind::AmpersandAmpersand
    };
    ("&&=") => {
        $crate::token::TokenKind::AmpersandAmpersandEqual
    };
    ("array") => {
        $crate::token::TokenKind::Array
    };
    ("(array)") => {
        $crate::token::TokenKind::ArrayCast
    };
    ("->") => {
        $crate::token::TokenKind::MinusGreaterThan
    };
    ("?->") => {
        $crate::token::TokenKind::QuestionMinusGreaterThan
    };
    ("@") => {
        $crate::token::TokenKind::At
    };
    ("as") => {
        $crate::token::TokenKind::As
    };
    ("*") => {
        $crate::token::TokenKind::Asterisk
    };
    ("#[") => {
        $crate::token::TokenKind::HashLeftBracket
    };
    ("!") => {
        $crate::token::TokenKind::Bang
    };
    ("!=") => {
        $crate::token::TokenKind::BangEqual
    };
    ("<>") => {
        $crate::token::TokenKind::LessThanGreaterThan
    };
    ("!==") => {
        $crate::token::TokenKind::BangEqualEqual
    };
    ("<=>") => {
        $crate::token::TokenKind::LessThanEqualGreaterThan
    };
    ("(bool)") => {
        $crate::token::TokenKind::BoolCast
    };
    ("(boolean)") => {
        $crate::token::TokenKind::BooleanCast
    };
    ("and") => {
        $crate::token::TokenKind::And
    };
    ("or") => {
        $crate::token::TokenKind::Or
    };
    ("break") => {
        $crate::token::TokenKind::Break
    };
    ("callable") => {
        $crate::token::TokenKind::Callable
    };
    ("^") => {
        $crate::token::TokenKind::Caret
    };
    ("^=") => {
        $crate::token::TokenKind::CaretEqual
    };
    ("case") => {
        $crate::token::TokenKind::Case
    };
    ("catch") => {
        $crate::token::TokenKind::Catch
    };
    ("class") => {
        $crate::token::TokenKind::Class
    };
    ("__CLASS__") => {
        $crate::token::TokenKind::ClassConstant
    };
    ("__TRAIT__") => {
        $crate::token::TokenKind::TraitConstant
    };
    ("__FUNCTION__") => {
        $crate::token::TokenKind::FunctionConstant
    };
    ("__METHOD__") => {
        $crate::token::TokenKind::MethodConstant
    };
    ("__LINE__") => {
        $crate::token::TokenKind::LineConstant
    };
    ("__FILE__") => {
        $crate::token::TokenKind::FileConstant
    };
    ("clone") => {
        $crate::token::TokenKind::Clone
    };
    ("-=") => {
        $crate::token::TokenKind::MinusEqual
    };
    ("?>") => {
        $crate::token::TokenKind::CloseTag
    };
    ("??") => {
        $crate::token::TokenKind::QuestionQuestion
    };
    ("??=") => {
        $crate::token::TokenKind::QuestionQuestionEqual
    };
    ("*=") => {
        $crate::token::TokenKind::AsteriskEqual
    };
    (":") => {
        $crate::token::TokenKind::Colon
    };
    (",") => {
        $crate::token::TokenKind::Comma
    };
    ("// comment") => {
        $crate::token::TokenKind::SingleLineComment
    };
    ("# comment") => {
        $crate::token::TokenKind::HashComment
    };
    ("/* comment */") => {
        $crate::token::TokenKind::MultiLineComment
    };
    ("/** comment */") => {
        $crate::token::TokenKind::DocBlockComment
    };
    ("const") => {
        $crate::token::TokenKind::Const
    };
    ("continue") => {
        $crate::token::TokenKind::Continue
    };
    ("declare") => {
        $crate::token::TokenKind::Declare
    };
    ("--") => {
        $crate::token::TokenKind::MinusMinus
    };
    ("default") => {
        $crate::token::TokenKind::Default
    };
    ("__DIR__") => {
        $crate::token::TokenKind::DirConstant
    };
    ("/=") => {
        $crate::token::TokenKind::SlashEqual
    };
    ("do") => {
        $crate::token::TokenKind::Do
    };
    ("${") => {
        $crate::token::TokenKind::DollarLeftBrace
    };
    (".") => {
        $crate::token::TokenKind::Dot
    };
    (".=") => {
        $crate::token::TokenKind::DotEqual
    };
    ("=>") => {
        $crate::token::TokenKind::EqualGreaterThan
    };
    ("(double)") => {
        $crate::token::TokenKind::DoubleCast
    };
    ("(real)") => {
        $crate::token::TokenKind::RealCast
    };
    ("(float)") => {
        $crate::token::TokenKind::FloatCast
    };
    ("::") => {
        $crate::token::TokenKind::ColonColon
    };
    ("==") => {
        $crate::token::TokenKind::EqualEqual
    };
    ("\"") => {
        $crate::token::TokenKind::DoubleQuote
    };
    ("else") => {
        $crate::token::TokenKind::Else
    };
    ("echo") => {
        $crate::token::TokenKind::Echo
    };
    ("...") => {
        $crate::token::TokenKind::DotDotDot
    };
    ("elseif") => {
        $crate::token::TokenKind::ElseIf
    };
    ("empty") => {
        $crate::token::TokenKind::Empty
    };
    ("enddeclare") => {
        $crate::token::TokenKind::EndDeclare
    };
    ("endfor") => {
        $crate::token::TokenKind::EndFor
    };
    ("endforeach") => {
        $crate::token::TokenKind::EndForeach
    };
    ("endif") => {
        $crate::token::TokenKind::EndIf
    };
    ("endswitch") => {
        $crate::token::TokenKind::EndSwitch
    };
    ("endwhile") => {
        $crate::token::TokenKind::EndWhile
    };
    ("enum") => {
        $crate::token::TokenKind::Enum
    };
    ("=") => {
        $crate::token::TokenKind::Equal
    };
    ("extends") => {
        $crate::token::TokenKind::Extends
    };
    ("false") => {
        $crate::token::TokenKind::False
    };
    ("final") => {
        $crate::token::TokenKind::Final
    };
    ("finally") => {
        $crate::token::TokenKind::Finally
    };
    ("fn") => {
        $crate::token::TokenKind::Fn
    };
    ("for") => {
        $crate::token::TokenKind::For
    };
    ("foreach") => {
        $crate::token::TokenKind::Foreach
    };
    ("\\Fully\\Qualified\\Identifier") => {
        $crate::token::TokenKind::FullyQualifiedIdentifier
    };
    ("function") => {
        $crate::token::TokenKind::Function
    };
    ("goto") => {
        $crate::token::TokenKind::Goto
    };
    (">") => {
        $crate::token::TokenKind::GreaterThan
    };
    (">=") => {
        $crate::token::TokenKind::GreaterThanEqual
    };
    ("Identifier") => {
        $crate::token::TokenKind::Identifier
    };
    ("if") => {
        $crate::token::TokenKind::If
    };
    ("implements") => {
        $crate::token::TokenKind::Implements
    };
    ("include") => {
        $crate::token::TokenKind::Include
    };
    ("include_once") => {
        $crate::token::TokenKind::IncludeOnce
    };
    ("++") => {
        $crate::token::TokenKind::PlusPlus
    };
    ("instanceof") => {
        $crate::token::TokenKind::Instanceof
    };
    ("insteadof") => {
        $crate::token::TokenKind::Insteadof
    };
    ("exit") => {
        $crate::token::TokenKind::Exit
    };
    ("unset") => {
        $crate::token::TokenKind::Unset
    };
    ("isset") => {
        $crate::token::TokenKind::Isset
    };
    ("list") => {
        $crate::token::TokenKind::List
    };
    ("(int)") => {
        $crate::token::TokenKind::IntCast
    };
    ("(integer)") => {
        $crate::token::TokenKind::IntegerCast
    };
    ("interface") => {
        $crate::token::TokenKind::Interface
    };
    ("{") => {
        $crate::token::TokenKind::LeftBrace
    };
    ("[") => {
        $crate::token::TokenKind::LeftBracket
    };
    ("(") => {
        $crate::token::TokenKind::LeftParenthesis
    };
    (")") => {
        $crate::token::TokenKind::RightParenthesis
    };
    ("<<") => {
        $crate::token::TokenKind::LeftShift
    };
    ("<<=") => {
        $crate::token::TokenKind::LeftShiftEqual
    };
    (">>") => {
        $crate::token::TokenKind::RightShift
    };
    (">>=") => {
        $crate::token::TokenKind::RightShiftEqual
    };
    ("<") => {
        $crate::token::TokenKind::LessThan
    };
    ("<=") => {
        $crate::token::TokenKind::LessThanEqual
    };
    ("match") => {
        $crate::token::TokenKind::Match
    };
    ("-") => {
        $crate::token::TokenKind::Minus
    };
    ("namespace") => {
        $crate::token::TokenKind::Namespace
    };
    ("\\") => {
        $crate::token::TokenKind::NamespaceSeparator
    };
    ("__NAMESPACE__") => {
        $crate::token::TokenKind::NamespaceConstant
    };
    ("new") => {
        $crate::token::TokenKind::New
    };
    ("null") => {
        $crate::token::TokenKind::Null
    };
    ("(object)") => {
        $crate::token::TokenKind::ObjectCast
    };
    ("(unset)") => {
        $crate::token::TokenKind::UnsetCast
    };
    ("<?php") => {
        $crate::token::TokenKind::OpenTag
    };
    ("<?=") => {
        $crate::token::TokenKind::EchoTag
    };
    ("<?") => {
        $crate::token::TokenKind::ShortOpenTag
    };
    ("%") => {
        $crate::token::TokenKind::Percent
    };
    ("%=") => {
        $crate::token::TokenKind::PercentEqual
    };
    ("|") => {
        $crate::token::TokenKind::Pipe
    };
    ("|=") => {
        $crate::token::TokenKind::PipeEqual
    };
    ("+") => {
        $crate::token::TokenKind::Plus
    };
    ("+=") => {
        $crate::token::TokenKind::PlusEqual
    };
    ("**") => {
        $crate::token::TokenKind::AsteriskAsterisk
    };
    ("**=") => {
        $crate::token::TokenKind::AsteriskAsteriskEqual
    };
    ("private(set)") => {
        $crate::token::TokenKind::PrivateSet
    };
    ("private") => {
        $crate::token::TokenKind::Private
    };
    ("protected") => {
        $crate::token::TokenKind::Protected
    };
    ("protected(set)") => {
        $crate::token::TokenKind::ProtectedSet
    };
    ("public") => {
        $crate::token::TokenKind::Public
    };
    ("public(set)") => {
        $crate::token::TokenKind::PublicSet
    };
    ("Qualified\\Identifier") => {
        $crate::token::TokenKind::QualifiedIdentifier
    };
    ("?") => {
        $crate::token::TokenKind::Question
    };
    ("?:") => {
        $crate::token::TokenKind::QuestionColon
    };
    ("require") => {
        $crate::token::TokenKind::Require
    };
    ("require_once") => {
        $crate::token::TokenKind::RequireOnce
    };
    ("return") => {
        $crate::token::TokenKind::Return
    };
    ("}") => {
        $crate::token::TokenKind::RightBrace
    };
    ("]") => {
        $crate::token::TokenKind::RightBracket
    };
    (";") => {
        $crate::token::TokenKind::Semicolon
    };
    ("/") => {
        $crate::token::TokenKind::Slash
    };
    ("static") => {
        $crate::token::TokenKind::Static
    };
    ("(string)") => {
        $crate::token::TokenKind::StringCast
    };
    ("(binary)") => {
        $crate::token::TokenKind::BinaryCast
    };
    ("switch") => {
        $crate::token::TokenKind::Switch
    };
    ("throw") => {
        $crate::token::TokenKind::Throw
    };
    ("trait") => {
        $crate::token::TokenKind::Trait
    };
    ("===") => {
        $crate::token::TokenKind::EqualEqualEqual
    };
    ("true") => {
        $crate::token::TokenKind::True
    };
    ("try") => {
        $crate::token::TokenKind::Try
    };
    ("use") => {
        $crate::token::TokenKind::Use
    };
    ("var") => {
        $crate::token::TokenKind::Var
    };
    ("$variable") => {
        $crate::token::TokenKind::Variable
    };
    ("yield") => {
        $crate::token::TokenKind::Yield
    };
    ("while") => {
        $crate::token::TokenKind::While
    };
    ("~") => {
        $crate::token::TokenKind::Tilde
    };
    ("||") => {
        $crate::token::TokenKind::PipePipe
    };
    ("xor") => {
        $crate::token::TokenKind::Xor
    };
    ($name:ident) => {
        $crate::token::TokenKind::$name
    };
    ($first:tt | $($rest:tt)+) => {
        $crate::T![$first] | $crate::T![$($rest)+]
    };
    ($($kind:tt),+ $(,)?) => {
        &[$($crate::T![$kind]),+]
    };
}
