//! This file defines the various tokens requried by the Enso lexer.
//!
//! This file makes heavy use of terminology from the Enso design documentation, particularly the
//! [syntax](https://dev.enso.org/docs/enso/syntax) documentation. For the sake of brevity, many
//! terms will _not_ be defined here.

use crate::prelude::*;



// =============
// === Token ===
// =============

/// A lexer token.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Token {
    /// The shape of the token.
    pub shape : Shape,
    /// The length (in characters) of this token.
    pub length : usize,
    /// The number of trailing spaces after this token before the next.
    pub offset : usize,
}

impl Token {
    /// Get the length that the token takes up in the program source.
    pub fn source_length(&self) -> usize {
        self.length + self.offset
    }
}

/// Constructors for the various forms of token.
#[allow(non_snake_case)]
impl Token {
    /// Construct a token representing a referent identifier.
    pub fn Referent(name:impl Str, offset:usize) -> Token {
        let str    = name.into();
        let length = str.chars().count();
        let shape  = Shape::Referent(str);
        Token{shape,length,offset}
    }

    /// Construct a token representing a variable identifier.
    pub fn Variable(name:impl Str, offset:usize) -> Token {
        let str    = name.into();
        let length = str.chars().count();
        let shape  = Shape::Variable(str);
        Token{shape,length,offset}
    }

    /// Construct a token representing an external identifier.
    pub fn External(name:impl Str, offset:usize) -> Token {
        let str    = name.into();
        let length = str.chars().count();
        let shape  = Shape::External(str);
        Token{shape,length,offset}
    }

    /// Construct a token representing a blank identifier.
    pub fn Blank(offset:usize) -> Token {
        let shape  = Shape::Blank;
        let length = 1;
        Token{shape,length,offset}
    }

    /// Construct a token representing an operator.
    pub fn Operator(name:impl Str, offset:usize) -> Token {
        let str    = name.into();
        let length = str.chars().count();
        let shape  = Shape::Operator(str);
        Token{shape,length,offset}
    }

    /// Construct a token representing a modifier operator.
    pub fn Modifier(name:impl Str, offset:usize) -> Token {
        let str    = name.into();
        let length = str.chars().count() + 1;
        let shape  = Shape::Modifier(str);
        Token{shape,length,offset}
    }

    /// Construct a token representing a number literal.
    pub fn Number(base:impl Str, num:impl Into<String>, offset:usize) -> Token {
        let str      = num.into();
        let base_str = base.into();
        let length   = if base_str.is_empty() {
            str.chars().count()
        } else {
            base_str.chars().count() + 1 + str.chars().count()
        };
        let shape = Shape::Number{base:base_str,number:str};
        Token{shape,length,offset}
    }

    /// Construct a token representing a dangling number base.
    pub fn DanglingBase(base:impl Str, offset:usize) -> Token {
        let base_str = base.into();
        let length   = base_str.chars().count() + 1;
        let shape    = Shape::DanglingBase(base_str);
        Token{shape,length,offset}
    }

    /// Construct a token representing a line of text.
    pub fn TextLine(style:TextStyle, segments:Vec<Token>, offset:usize) -> Token {
        let length = style.length() + segments.iter().fold(0,|l,r| l + r.length + r.offset);
        let shape = Shape::TextLine{style,segments};
        Token{shape,length,offset}
    }

    /// Construct a token representing a block of text.
    pub fn TextBlock(style:TextStyle, lines:Vec<Token>, indent: usize, offset:usize) -> Token {
        let length = style.length() + lines.iter().fold(0, |l,r| {
            l + match r.shape {
                Shape::Line {..}    => indent + r.length + r.offset,
                Shape::BlankLine(_) => indent + r.length + r.offset,
                _                   => unreachable_panic!("Text blocks should only contain lines."),
            }
        });
        let shape = Shape::TextBlock{style,lines};
        Token{shape,length,offset}
    }

    /// Construct a token representing a raw text segment.
    pub fn TextSegmentRaw(str:impl Str, offset:usize) -> Token {
        let string = str.into();
        let length = string.len();
        let shape  = Shape::TextSegmentRaw(string);
        Token{shape,length,offset}
    }

    /// Construct a token representing an escape sequence.
    pub fn TextSegmentEscape(repr:impl Str, offset:usize) -> Token {
        let string = repr.into();
        let length = string.len();
        let shape  = Shape::TextSegmentEscape(string);
        Token{shape,length,offset}
    }

    /// Construct a token representing an interpolated text segment.
    pub fn TextSegmentInterpolate(tokens:Vec<Token>, offset:usize) -> Token {
        let length_of_interpolation_ticks = 2;
        let length =
            length_of_interpolation_ticks + tokens.iter().fold(0,|l,r| l + r.length + r.offset);
        let shape = Shape::TextSegmentInterpolate{tokens};
        Token{shape,length,offset}
    }

    /// Construct a token representing an invalid text segment.
    pub fn TextSegmentInvalid(str:impl Str, length:usize, offset:usize) -> Token {
        let string = str.into();
        let shape  = Shape::TextSegmentInvalid(string);
        Token{shape,length,offset}
    }

    /// Construct a token representing a line of tokens.
    pub fn Line(tokens:Vec<Token>, offset:usize, trailing_line_ending:LineEnding) -> Token {
        let line_ending_len = trailing_line_ending.size();
        let length          = tokens.iter().fold(line_ending_len,|l,r| l + r.offset + r.length);
        let shape           = Shape::Line{tokens,trailing_line_ending};
        Token{shape,length,offset}
    }

    /// Construct a token representing a blank line.
    ///
    /// The `offset` for blank lines is from the leftmost column, not from the parent block's
    /// indentation.
    pub fn BlankLine(offset:usize, trailing_line_ending:LineEnding) -> Token {
        let length = trailing_line_ending.size();
        let shape  = Shape::BlankLine(trailing_line_ending);
        Token{shape,length,offset}
    }

    /// Construct a token representing a block.
    pub fn Block
    ( block_type  : BlockType
    , indent      : usize
    , lines       : Vec<Token>
    , offset      : usize
    ) -> Token {
        let length = lines.iter().map(|line| {
            let line_length = line.length;
            let line_offset = line.offset;
            match line.shape {
                Shape::Line{..}     => indent + line_offset + line_length,
                Shape::BlankLine(_) => line_offset + line_length,
                _                   => unreachable_panic!("Tokens in a blocks should always be lines."),
            }
        }).sum();
        let shape = Shape::Block{block_type,indent,lines};
        Token{shape,length,offset}
    }

    /// Construct a token representing an invalid suffix.
    pub fn InvalidSuffix(text:impl Str, offset:usize) -> Token {
        let str    = text.into();
        let length = str.chars().count();
        let shape  = Shape::InvalidSuffix(str);
        Token{shape,length,offset}
    }

    /// Construct a token representing an unrecognised lexeme.
    pub fn Unrecognized(text:impl Str, offset:usize) -> Token {
        let str    = text.into();
        let length = str.chars().count();
        let shape  = Shape::Unrecognized(str);
        Token{shape,length,offset}
    }
}



// =================
// === BlockType ===
// =================

/// The type for an Enso Block token.
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum BlockType {
    /// A block made up of arguments to a function.
    Continuous,
    /// A block made up of separate lines.
    Discontinuous,
}



// ==================
// === LineEnding ===
// ==================

/// The type of newline associated with the line.
#[derive(Copy,Clone,Debug,Display,PartialEq,Eq)]
pub enum LineEnding {
    /// There is no newline.
    None,
    /// The unix-style line-feed (`'\n'`),
    LF,
    /// The windows-style carriage-return, line-feed (`"\r\n"`).
    CRLF
}

impl LineEnding {
    /// Get the number of rust `char`s that the newline type takes up.
    pub fn size(self) -> usize {
        match self {
            Self::None => 0,
            Self::LF   => 1,
            Self::CRLF => 2,
        }
    }
}


// === Trait Impls ===

impl Default for LineEnding {
    fn default() -> Self {
        LineEnding::None
    }
}



// =================
// === TextStyle ===
// =================

/// The style of the text literal.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum TextStyle {
    // === Line ===

    /// A interpolated text line literal.
    InterpolatedLine,
    /// A raw text line literal.
    RawLine,
    /// An unclosed text line literal.
    UnclosedLine,

    // === Block ===

    /// An interpolated text block literal.
    InterpolatedBlock,
    /// A raw text block literal.
    RawBlock,
}

impl TextStyle {
    /// Calculate the length of the delimiters for a particular style of text literal.
    pub fn length(&self) -> usize {
        match self {
            TextStyle::InterpolatedLine  => 2,
            TextStyle::RawLine           => 2,
            TextStyle::UnclosedLine      => 1,
            TextStyle::InterpolatedBlock => 3,
            TextStyle::RawBlock          => 3,
        }
    }
}



// =============
// === Shape ===
// =============

/// The shapes of tokens needed by the Enso lexer.
///
/// This is a very small set of shapes, because the [`Token`] type only deals with the tokens that
/// the lexer works with, not the full complexity of Enso's syntax.
#[allow(missing_docs)]
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Shape {
    // === Identifiers ===

    /// An identifier in referent form.
    Referent(String),
    /// An identifier in variable form.
    Variable(String),
    /// An identifier not conforming to the Enso identifier rules (e.g. a Java identifier).
    External(String),
    /// A blank identifier (`_`).
    Blank,
    /// An operator identifier.
    Operator(String),
    /// A modifier identifier.
    Modifier(String),

    // === Literals ===

    /// A literal number.
    Number{base:String, number:String},
    /// A dangling base from a number literal.
    DanglingBase(String),
    /// A text line literal.
    TextLine{
        /// The type of literal being encoded.
        style : TextStyle,
        /// The segments that make up the line of text.
        segments : Vec<Token>,
    },
    /// A text block literal.
    TextBlock{
        /// The type of literal being encoded.
        style : TextStyle,
        /// The lines in the text block literal.
        lines : Vec<Token>
    },
    /// A segment of a line of text containing only literal text.
    TextSegmentRaw(String),
    /// A segment of a line of text that represents an escape sequence.
    TextSegmentEscape(String),
    /// A segment of a line of text that contains an interpolated expression.
    TextSegmentInterpolate {
        /// The tokens making up the interpolated expression.
        tokens : Vec<Token>
    },
    /// An invalid text segment (e.g. unclosed interpolate segment).
    TextSegmentInvalid(String),

    // === Lines ===
    /// A line containing tokens.
    ///
    /// The offset for a line is always zero, as it is contained in a block with a defined
    /// indentation.
    Line{
        /// The tokens on the line.
        tokens : Vec<Token>,
        /// The line ending that _ends_ the line.
        ///
        /// Please note that the concept of 'ending' the line is a bit strange, as blocks are
        /// treated as tokens in their own right, and hence are included in lines.
        trailing_line_ending : LineEnding
    },
    /// A blank line.
    ///
    /// The offset for a blank line is from the leftmost column, as it may be negative from the
    /// block's indentation level.
    BlankLine(LineEnding),

    // === Block ===
    /// A block of tokens.
    Block {
        /// The type of the block.
        block_type : BlockType,
        /// The leading indentation of the block.
        indent : usize,
        /// The lines in the block.
        lines : Vec<Token>,
    },

    // === Errors ===
    /// An invalid suffix.
    InvalidSuffix(String),
    /// An unrecognized token.
    Unrecognized(String),
}

impl Shape {

    /// Construct an identifier in referent form.
    pub fn referent(name:impl Into<String>) -> Shape {
        Shape::Referent(name.into())
    }

    /// Construct an identifier in variable form.
    pub fn variable(name:impl Into<String>) -> Shape {
        Shape::Variable(name.into())
    }

    /// Construct an identifier in external form.
    pub fn external(name:impl Into<String>) -> Shape {
        Shape::External(name.into())
    }

    /// Construct a blank identifier.
    ///
    /// This is provided as a function for completeness.
    pub fn blank() -> Shape {
        Shape::Blank
    }

    /// Construct an operator identifier.
    pub fn operator(opr:impl Into<String>) -> Shape {
        Shape::Operator(opr.into())
    }

    /// Construct a modifier identifier.
    pub fn modifier(opr:impl Into<String>) -> Shape {
        Shape::Modifier(opr.into())
    }

    /// Construct a number literal.
    pub fn number(base:impl Into<String>, num:impl Into<String>) -> Shape {
        Shape::Number{base:base.into(),number:num.into()}
    }

    /// Construct a dangling base literal.
    pub fn dangling_base(base:impl Into<String>) -> Shape {
        Shape::DanglingBase(base.into())
    }

    /// Construct a text line literal.
    pub fn text_line(style:TextStyle, segments:Vec<Token>) -> Shape {
        Shape::TextLine{style,segments}
    }

    /// Construct a text block literal.
    pub fn text_block(style:TextStyle, lines:Vec<Token>) -> Shape {
        Shape::TextBlock{style,lines}
    }

    /// Construct a raw text segment.
    pub fn text_segment_raw(text:impl Str) -> Shape {
        Shape::TextSegmentRaw(text.into())
    }

    /// Construct a text segment containing an escape sequence.
    pub fn text_segment_escape(repr:impl Str) -> Shape {
        Shape::TextSegmentEscape(repr.into())
    }

    /// Construct a text segment containing an interpolated expression.
    pub fn text_segment_interpolate(tokens:Vec<Token>) -> Shape {
        Shape::TextSegmentInterpolate {tokens}
    }

    /// Construct an invalid text segment.
    pub fn text_segment_invalid(str:impl Str) -> Shape {
        Shape::TextSegmentInvalid(str.into())
    }

    /// Construct a line that contains tokens.
    pub fn line(tokens:Vec<Token>, trailing_line_ending:LineEnding) -> Shape {
        Shape::Line{tokens,trailing_line_ending }
    }

    /// Construct a line that is blank.
    pub fn blank_line(trailing_line_ending:LineEnding) -> Shape {
        Shape::BlankLine(trailing_line_ending)
    }

    /// Construct a block containing lines.
    pub fn block(block_type:BlockType, indent:usize, lines:Vec<Token>) -> Shape {
        Shape::Block{block_type,indent,lines}
    }

    /// Construct an invalid suffix.
    pub fn invalid_suffix(text:impl Into<String>) -> Shape {
        Shape::InvalidSuffix(text.into())
    }

    /// Construct an unrecognised token.
    pub fn unrecognized(text:impl Into<String>) -> Shape {
        Shape::Unrecognized(text.into())
    }
}



// ==============
// === Stream ===
// ==============

/// A representation of the Enso token stream.
#[derive(Clone,Debug,Default,PartialEq)]
pub struct Stream {
    /// The tokens in the token stream.
    tokens:Vec<Token>
}

impl Stream {
    /// Append the provided `token` to the token stream.
    pub fn append(&mut self, token:Token) {
        self.tokens.push(token)
    }

    /// Get a reference to the tokens in the stream.
    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    /// Get the length of the elements in the token stream.
    pub fn tokens_len(&self) -> usize {
        self.tokens.iter().map(|token|token.length + token.offset).sum()
    }
}

impl Deref for Stream {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

impl DerefMut for Stream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tokens
    }
}


// === Trait Impls ===

impl From<Vec<Token>> for Stream {
    fn from(tokens:Vec<Token>) -> Self {
        Stream{tokens}
    }
}

impl Into<Vec<Token>> for Stream {
    fn into(self) -> Vec<Token> {
        self.tokens
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;


    // === Testing Utilities ===

    /// Asserts that the `token` has the provided `shape`.
    pub fn assert_shape(token:&Token, shape:Shape) {
        assert_eq!(token.shape,shape);
    }

    /// Asserts that the `token` has the provided `length`.
    pub fn assert_length(token:&Token, length:usize) {
        assert_eq!(token.length,length)
    }


    // === Tests for Token Construction ===

    #[test]
    fn construct_referent_token() {
        let token = Token::Referent("Some_Ref_Name",0);
        assert_shape(&token,Shape::referent("Some_Ref_Name"));
        assert_length(&token,13);
    }

    #[test]
    fn construct_variable_token() {
        let token = Token::Variable("some_variable_name",0);
        assert_shape(&token,Shape::variable("some_variable_name"));
        assert_length(&token,18);
    }

    #[test]
    fn construct_external_name_token() {
        let token = Token::External("camelCase",0);
        assert_shape(&token,Shape::external("camelCase"));
        assert_length(&token,9);
    }

    #[test]
    fn construct_blank_token() {
        let token = Token::Blank(0);
        assert_shape(&token,Shape::blank());
        assert_length(&token,1);
    }

    #[test]
    fn construct_operator_token() {
        let token = Token::Operator("==>",0);
        assert_shape(&token,Shape::operator("==>"));
        assert_length(&token,3);
    }

    #[test]
    fn construct_modifier_token() {
        let token = Token::Modifier("+",0);
        assert_shape(&token,Shape::modifier("+"));
        assert_length(&token,2);
    }

    #[test]
    fn construct_number_token() {
        let token = Token::Number("","1231",0);
        assert_shape(&token,Shape::number("","1231"));
        assert_length(&token,4);
    }

    #[test]
    fn construct_dangling_base_token() {
        let token = Token::DanglingBase("15",0);
        assert_shape(&token,Shape::dangling_base("15"));
        assert_length(&token,3);
    }

    // TODO [AA] Tests for the text tokens.

    #[test]
    fn construct_text_line_token() {
        let token = Token::TextLine(TextStyle::RawLine,vec![],0);
        assert_shape(&token,Shape::text_line(TextStyle::RawLine,vec![]));
        assert_length(&token,2);
    }

    #[test]
    fn construct_text_block_token() {
        let lines = vec![
            Token::Line(vec![Token::TextSegmentRaw("foo",0)],0,LineEnding::LF),
            Token::Line(vec![Token::TextSegmentInterpolate(vec![],0)],0,LineEnding::LF)
        ];
        let token = Token::TextBlock(TextStyle::InterpolatedBlock,lines.clone(),2,0);
        assert_shape(&token,Shape::text_block(TextStyle::InterpolatedBlock,lines.clone()));
        assert_length(&token,14);
    }

    #[test]
    fn construct_text_segment_raw_token() {
        let token = Token::TextSegmentRaw("FooBar Baz Bam",0);
        assert_shape(&token,Shape::text_segment_raw("FooBar Baz Bam"));
        assert_length(&token,14);
    }

    #[test]
    fn construct_text_segment_escape() {
        let token = Token::TextSegmentEscape("\\t",0);
        assert_shape(&token,Shape::text_segment_escape("\\t"));
        assert_length(&token,2);
    }

    #[test]
    fn construct_text_segment_interpolate() {
        let tokens = vec![
            Token::Variable("foo",0),
            Token::Operator("+",1),
            Token::Number("","1",1)
        ];
        let token = Token::TextSegmentInterpolate(tokens.clone(),0);
        assert_shape(&token,Shape::text_segment_interpolate(tokens.clone()));
        assert_length(&token,9);
    }

    #[test]
    fn construct_text_segment_invalid() {
        let token = Token::TextSegmentInvalid("scream",12,0);
        assert_shape(&token,Shape::text_segment_invalid("scream"));
        assert_length(&token,12);
    }

    #[test]
    fn construct_line_token() {
        let tokens = vec![Token::Variable("aa",0),Token::Referent("Abc",1)];
        let token  = Token::Line(tokens.clone(), 4, LineEnding::LF);
        assert_shape(&token,Shape::line(tokens.clone(), LineEnding::LF));
        assert_length(&token,7);
    }

    #[test]
    fn construct_blank_line_token() {
        let token = Token::BlankLine(13,LineEnding::LF);
        assert_shape(&token, Shape::blank_line(LineEnding::LF));
        assert_length(&token,1);
    }

    #[test]
    fn construct_block_token_lf() {
        let lines = vec![
            Token::Line(vec![],0,LineEnding::LF),
            Token::Line(vec![],4,LineEnding::LF)
        ];
        let token = Token::Block(BlockType::Continuous,4,lines.clone(),0);
        assert_shape(&token,Shape::block(BlockType::Continuous,4,lines.clone()));
        assert_length(&token,14);
    }

    #[test]
    fn construct_block_token_crlf() {
        let lines = vec![
            Token::Line(vec![],0,LineEnding::CRLF),
            Token::Line(vec![],4,LineEnding::CRLF)
        ];
        let token = Token::Block(BlockType::Continuous,4,lines.clone(),0);
        assert_shape(&token,Shape::block(BlockType::Continuous,4,lines.clone()));
        assert_length(&token,16);
    }

    #[test]
    fn construct_invalid_suffix_token() {
        let token = Token::InvalidSuffix("aaa",0);
        assert_shape(&token,Shape::invalid_suffix("aaa"));
        assert_length(&token,3);
    }

    #[test]
    fn construct_unrecognized_token() {
        let token = Token::Unrecognized("a",0);
        assert_shape(&token,Shape::unrecognized("a"));
        assert_length(&token,1);
    }
}
