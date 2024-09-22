#![allow(non_snake_case)]

use std::{cmp::Ordering, str::Utf8Error};

use anyhow::Result;
use lsp_types::{Position, Range};

pub use derive::{DocumentNode, ParsableTag};

use crate::{
    grammar::{TagAttributeType, TagDefinition},
    spel::{
        self,
        ast::{SpelAst, SpelResult},
    },
};

pub(crate) trait DocumentNode {
    fn range(&self) -> Range;
}

pub(crate) trait ParsableTag {
    // TODO: eh?
    fn parse(parser: &mut TreeParser) -> Result<ParsedTag<Self>>
    where
        Self: Sized,
        Self: Tag;

    fn definition(&self) -> TagDefinition;

    fn open_location(&self) -> &Location;

    fn close_location(&self) -> &Location;

    fn body(&self) -> &Option<TagBody>;

    fn spel_attributes(&self) -> Vec<(&str, &ParsedAttribute<SpelAttribute>)>;

    fn spel_attribute(&self, name: &str) -> Option<&ParsedAttribute<SpelAttribute>>;
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tree {
    pub(crate) header: Header,
    pub(crate) nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParsedNode<R, E> {
    Valid(R),
    Incomplete(E),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParsedLocation {
    Valid(Location),
    Erroneous(Location), // TODO: needs info about error
    Missing,
}

impl ParsedLocation {
    pub(crate) fn location(&self) -> Option<&Location> {
        return match &self {
            ParsedLocation::Valid(location) => Some(location),
            ParsedLocation::Erroneous(location) => Some(location),
            ParsedLocation::Missing => None,
        };
    }
}

pub(crate) trait RangedNode {
    fn range(&self) -> Option<Range>;
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Header {
    pub(crate) java_headers: Vec<ParsedNode<PageHeader, IncompletePageHeader>>,
    pub(crate) taglib_imports: Vec<ParsedNode<TagLibImport, IncompleteTagLibImport>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PageHeader {
    pub(crate) open_bracket: Location,
    pub(crate) page: Location,
    pub(crate) language: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) page_encoding: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) content_type: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) imports: Vec<ParsedAttribute<PlainAttribute>>,
    pub(crate) close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IncompletePageHeader {
    pub(crate) open_bracket: ParsedLocation,
    pub(crate) page: ParsedLocation,
    pub(crate) language: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) page_encoding: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) content_type: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) imports: Vec<ParsedAttribute<PlainAttribute>>,
    pub(crate) close_bracket: ParsedLocation,
}

impl RangedNode for IncompletePageHeader {
    fn range(&self) -> Option<Range> {
        let start = self
            .open_bracket
            .location()
            .or_else(|| self.page.location())
            .or_else(|| self.language.as_ref().map(|a| a.start()))
            .or_else(|| self.page_encoding.as_ref().map(|a| a.start()))
            .or_else(|| self.content_type.as_ref().map(|a| a.start()))
            .or_else(|| self.imports.first().map(|a| a.start()))
            .or_else(|| self.close_bracket.location())?;
        let end = self
            .close_bracket
            .location()
            .or_else(|| self.imports.last().map(|a| a.start()))
            .or_else(|| self.content_type.as_ref().map(|a| a.start()))
            .or_else(|| self.page_encoding.as_ref().map(|a| a.start()))
            .or_else(|| self.language.as_ref().map(|a| a.start()))
            .or_else(|| self.page.location())
            .or_else(|| self.open_bracket.location())?;
        return Some(Range {
            start: Position {
                character: start.char as u32,
                line: start.line as u32,
            },
            end: Position {
                character: (end.char + end.length) as u32,
                line: end.line as u32,
            },
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagLibImport {
    pub(crate) open_bracket: Location,
    pub(crate) taglib: Location,
    pub(crate) origin: TagLibOrigin,
    pub(crate) prefix: ParsedAttribute<PlainAttribute>,
    pub(crate) close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IncompleteTagLibImport {
    pub(crate) open_bracket: ParsedLocation,
    pub(crate) taglib: ParsedLocation,
    pub(crate) origin: Option<TagLibOrigin>,
    pub(crate) prefix: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) close_bracket: ParsedLocation,
    pub(crate) errors: Vec<ErrorNode>,
}

impl RangedNode for IncompleteTagLibImport {
    fn range(&self) -> Option<Range> {
        let start = self
            .open_bracket
            .location()
            .or_else(|| self.taglib.location())
            .or_else(|| self.origin.as_ref().map(|e| e.start()))
            .or_else(|| self.prefix.as_ref().map(|e| e.start()))
            .or_else(|| self.close_bracket.location())?;
        let end = self
            .close_bracket
            .location()
            .or_else(|| self.prefix.as_ref().map(|e| e.end()))
            .or_else(|| self.origin.as_ref().map(|e| e.end()))
            .or_else(|| self.taglib.location())
            .or_else(|| self.open_bracket.location())?;
        return Some(Range {
            start: Position {
                character: start.char as u32,
                line: start.line as u32,
            },
            end: Position {
                character: (end.char + end.length) as u32,
                line: end.line as u32,
            },
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagLibOrigin {
    Uri(ParsedAttribute<PlainAttribute>),
    TagDir(ParsedAttribute<PlainAttribute>),
}

impl TagLibOrigin {
    fn start(&self) -> &Location {
        return match &self {
            TagLibOrigin::Uri(uri) => uri.start(),
            TagLibOrigin::TagDir(tagdir) => tagdir.start(),
        };
    }

    fn end(&self) -> &Location {
        return match &self {
            TagLibOrigin::Uri(uri) => uri.end(),
            TagLibOrigin::TagDir(tagdir) => tagdir.end(),
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagBody {
    pub(crate) open_location: Location,
    pub(crate) nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) enum Node {
    Tag(ParsedTag<SpmlTag>),
    Html(HtmlNode),
    Text(TextNode),
    Error(ErrorNode),
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) struct HtmlNode {
    pub(crate) open_location: Location,
    pub(crate) name: String,
    pub(crate) attributes: Vec<ParsedAttribute<PlainAttribute>>,
    pub(crate) body: Option<TagBody>,
    pub(crate) close_location: Location,
}

impl HtmlNode {
    pub(crate) fn open_location(&self) -> &Location {
        return &self.open_location;
    }

    pub(crate) fn close_location(&self) -> &Location {
        return &self.close_location;
    }

    pub(crate) fn body(&self) -> &Option<TagBody> {
        return &self.body;
    }
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) struct TextNode {
    pub(crate) content: String,
    pub(crate) range: Range,
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) struct ErrorNode {
    pub(crate) content: String,
    pub(crate) range: Range,
}

#[derive(Clone, Debug, PartialEq, DocumentNode, ParsableTag)]
pub(crate) enum SpmlTag {
    SpArgument(SpArgument),
    SpAttribute(SpAttribute),
    SpBarcode(SpBarcode),
    SpBreak(SpBreak),
    SpCalendarsheet(SpCalendarsheet),
    SpCheckbox(SpCheckbox),
    SpCode(SpCode),
    SpCollection(SpCollection),
    SpCondition(SpCondition),
    SpDiff(SpDiff),
    SpElse(SpElse),
    SpElseIf(SpElseIf),
    SpError(SpError),
    SpExpire(SpExpire),
    SpFilter(SpFilter),
    SpFor(SpFor),
    SpForm(SpForm),
    SpHidden(SpHidden),
    SpIf(SpIf),
    SpInclude(SpInclude),
    SpIo(SpIo),
    SpIterator(SpIterator),
    SpJson(SpJson),
    SpLinkedinformation(SpLinkedinformation),
    SpLinktree(SpLinktree),
    SpLivetree(SpLivetree),
    SpLog(SpLog),
    SpLogin(SpLogin),
    SpLoop(SpLoop),
    SpMap(SpMap),
    SpOption(SpOption),
    SpPassword(SpPassword),
    SpPrint(SpPrint),
    SpQuerytree(SpQuerytree),
    SpRadio(SpRadio),
    SpRange(SpRange),
    SpReturn(SpReturn),
    SpSass(SpSass),
    SpScaleimage(SpScaleimage),
    SpScope(SpScope),
    SpSearch(SpSearch),
    SpSelect(SpSelect),
    SpSet(SpSet),
    SpSort(SpSort),
    SpSubinformation(SpSubinformation),
    SpTagbody(SpTagbody),
    SpText(SpText),
    SpTextarea(SpTextarea),
    SpTextimage(SpTextimage),
    SpThrow(SpThrow),
    SpToggle(SpToggle),
    SpUpload(SpUpload),
    SpUrl(SpUrl),
    SpWarning(SpWarning),
    SpWorklist(SpWorklist),
    SpZip(SpZip),
    SptCounter(SptCounter),
    SptDate(SptDate),
    SptDiff(SptDiff),
    SptEmail2Img(SptEmail2Img),
    SptEncryptemail(SptEncryptemail),
    SptEscapeemail(SptEscapeemail),
    SptFormsolutions(SptFormsolutions),
    SptId2Url(SptId2Url),
    SptIlink(SptIlink),
    SptImageeditor(SptImageeditor),
    SptImp(SptImp),
    SptIterator(SptIterator),
    SptLink(SptLink),
    SptNumber(SptNumber),
    SptPersonalization(SptPersonalization),
    SptPrehtml(SptPrehtml),
    SptSmarteditor(SptSmarteditor),
    SptSpml(SptSpml),
    SptText(SptText),
    SptTextarea(SptTextarea),
    SptTimestamp(SptTimestamp),
    SptTinymce(SptTinymce),
    SptUpdown(SptUpdown),
    SptUpload(SptUpload),
    SptWorklist(SptWorklist),
}

// TODO: ...
impl Tag for SpmlTag {
    fn start(&self) -> &Location {
        return match &self {
            SpmlTag::SpArgument(tag) => tag.start(),
            SpmlTag::SpAttribute(tag) => tag.start(),
            SpmlTag::SpBarcode(tag) => tag.start(),
            SpmlTag::SpBreak(tag) => tag.start(),
            SpmlTag::SpCalendarsheet(tag) => tag.start(),
            SpmlTag::SpCheckbox(tag) => tag.start(),
            SpmlTag::SpCode(tag) => tag.start(),
            SpmlTag::SpCollection(tag) => tag.start(),
            SpmlTag::SpCondition(tag) => tag.start(),
            SpmlTag::SpDiff(tag) => tag.start(),
            SpmlTag::SpElse(tag) => tag.start(),
            SpmlTag::SpElseIf(tag) => tag.start(),
            SpmlTag::SpError(tag) => tag.start(),
            SpmlTag::SpExpire(tag) => tag.start(),
            SpmlTag::SpFilter(tag) => tag.start(),
            SpmlTag::SpFor(tag) => tag.start(),
            SpmlTag::SpForm(tag) => tag.start(),
            SpmlTag::SpHidden(tag) => tag.start(),
            SpmlTag::SpIf(tag) => tag.start(),
            SpmlTag::SpInclude(tag) => tag.start(),
            SpmlTag::SpIo(tag) => tag.start(),
            SpmlTag::SpIterator(tag) => tag.start(),
            SpmlTag::SpJson(tag) => tag.start(),
            SpmlTag::SpLinkedinformation(tag) => tag.start(),
            SpmlTag::SpLinktree(tag) => tag.start(),
            SpmlTag::SpLivetree(tag) => tag.start(),
            SpmlTag::SpLog(tag) => tag.start(),
            SpmlTag::SpLogin(tag) => tag.start(),
            SpmlTag::SpLoop(tag) => tag.start(),
            SpmlTag::SpMap(tag) => tag.start(),
            SpmlTag::SpOption(tag) => tag.start(),
            SpmlTag::SpPassword(tag) => tag.start(),
            SpmlTag::SpPrint(tag) => tag.start(),
            SpmlTag::SpQuerytree(tag) => tag.start(),
            SpmlTag::SpRadio(tag) => tag.start(),
            SpmlTag::SpRange(tag) => tag.start(),
            SpmlTag::SpReturn(tag) => tag.start(),
            SpmlTag::SpSass(tag) => tag.start(),
            SpmlTag::SpScaleimage(tag) => tag.start(),
            SpmlTag::SpScope(tag) => tag.start(),
            SpmlTag::SpSearch(tag) => tag.start(),
            SpmlTag::SpSelect(tag) => tag.start(),
            SpmlTag::SpSet(tag) => tag.start(),
            SpmlTag::SpSort(tag) => tag.start(),
            SpmlTag::SpSubinformation(tag) => tag.start(),
            SpmlTag::SpTagbody(tag) => tag.start(),
            SpmlTag::SpText(tag) => tag.start(),
            SpmlTag::SpTextarea(tag) => tag.start(),
            SpmlTag::SpTextimage(tag) => tag.start(),
            SpmlTag::SpThrow(tag) => tag.start(),
            SpmlTag::SpToggle(tag) => tag.start(),
            SpmlTag::SpUpload(tag) => tag.start(),
            SpmlTag::SpUrl(tag) => tag.start(),
            SpmlTag::SpWarning(tag) => tag.start(),
            SpmlTag::SpWorklist(tag) => tag.start(),
            SpmlTag::SpZip(tag) => tag.start(),
            SpmlTag::SptCounter(tag) => tag.start(),
            SpmlTag::SptDate(tag) => tag.start(),
            SpmlTag::SptDiff(tag) => tag.start(),
            SpmlTag::SptEmail2Img(tag) => tag.start(),
            SpmlTag::SptEncryptemail(tag) => tag.start(),
            SpmlTag::SptEscapeemail(tag) => tag.start(),
            SpmlTag::SptFormsolutions(tag) => tag.start(),
            SpmlTag::SptId2Url(tag) => tag.start(),
            SpmlTag::SptIlink(tag) => tag.start(),
            SpmlTag::SptImageeditor(tag) => tag.start(),
            SpmlTag::SptImp(tag) => tag.start(),
            SpmlTag::SptIterator(tag) => tag.start(),
            SpmlTag::SptLink(tag) => tag.start(),
            SpmlTag::SptNumber(tag) => tag.start(),
            SpmlTag::SptPersonalization(tag) => tag.start(),
            SpmlTag::SptPrehtml(tag) => tag.start(),
            SpmlTag::SptSmarteditor(tag) => tag.start(),
            SpmlTag::SptSpml(tag) => tag.start(),
            SpmlTag::SptText(tag) => tag.start(),
            SpmlTag::SptTextarea(tag) => tag.start(),
            SpmlTag::SptTimestamp(tag) => tag.start(),
            SpmlTag::SptTinymce(tag) => tag.start(),
            SpmlTag::SptUpdown(tag) => tag.start(),
            SpmlTag::SptUpload(tag) => tag.start(),
            SpmlTag::SptWorklist(tag) => tag.start(),
        };
    }

    fn end(&self) -> &Location {
        return match &self {
            SpmlTag::SpArgument(tag) => tag.end(),
            SpmlTag::SpAttribute(tag) => tag.end(),
            SpmlTag::SpBarcode(tag) => tag.end(),
            SpmlTag::SpBreak(tag) => tag.end(),
            SpmlTag::SpCalendarsheet(tag) => tag.end(),
            SpmlTag::SpCheckbox(tag) => tag.end(),
            SpmlTag::SpCode(tag) => tag.end(),
            SpmlTag::SpCollection(tag) => tag.end(),
            SpmlTag::SpCondition(tag) => tag.end(),
            SpmlTag::SpDiff(tag) => tag.end(),
            SpmlTag::SpElse(tag) => tag.end(),
            SpmlTag::SpElseIf(tag) => tag.end(),
            SpmlTag::SpError(tag) => tag.end(),
            SpmlTag::SpExpire(tag) => tag.end(),
            SpmlTag::SpFilter(tag) => tag.end(),
            SpmlTag::SpFor(tag) => tag.end(),
            SpmlTag::SpForm(tag) => tag.end(),
            SpmlTag::SpHidden(tag) => tag.end(),
            SpmlTag::SpIf(tag) => tag.end(),
            SpmlTag::SpInclude(tag) => tag.end(),
            SpmlTag::SpIo(tag) => tag.end(),
            SpmlTag::SpIterator(tag) => tag.end(),
            SpmlTag::SpJson(tag) => tag.end(),
            SpmlTag::SpLinkedinformation(tag) => tag.end(),
            SpmlTag::SpLinktree(tag) => tag.end(),
            SpmlTag::SpLivetree(tag) => tag.end(),
            SpmlTag::SpLog(tag) => tag.end(),
            SpmlTag::SpLogin(tag) => tag.end(),
            SpmlTag::SpLoop(tag) => tag.end(),
            SpmlTag::SpMap(tag) => tag.end(),
            SpmlTag::SpOption(tag) => tag.end(),
            SpmlTag::SpPassword(tag) => tag.end(),
            SpmlTag::SpPrint(tag) => tag.end(),
            SpmlTag::SpQuerytree(tag) => tag.end(),
            SpmlTag::SpRadio(tag) => tag.end(),
            SpmlTag::SpRange(tag) => tag.end(),
            SpmlTag::SpReturn(tag) => tag.end(),
            SpmlTag::SpSass(tag) => tag.end(),
            SpmlTag::SpScaleimage(tag) => tag.end(),
            SpmlTag::SpScope(tag) => tag.end(),
            SpmlTag::SpSearch(tag) => tag.end(),
            SpmlTag::SpSelect(tag) => tag.end(),
            SpmlTag::SpSet(tag) => tag.end(),
            SpmlTag::SpSort(tag) => tag.end(),
            SpmlTag::SpSubinformation(tag) => tag.end(),
            SpmlTag::SpTagbody(tag) => tag.end(),
            SpmlTag::SpText(tag) => tag.end(),
            SpmlTag::SpTextarea(tag) => tag.end(),
            SpmlTag::SpTextimage(tag) => tag.end(),
            SpmlTag::SpThrow(tag) => tag.end(),
            SpmlTag::SpToggle(tag) => tag.end(),
            SpmlTag::SpUpload(tag) => tag.end(),
            SpmlTag::SpUrl(tag) => tag.end(),
            SpmlTag::SpWarning(tag) => tag.end(),
            SpmlTag::SpWorklist(tag) => tag.end(),
            SpmlTag::SpZip(tag) => tag.end(),
            SpmlTag::SptCounter(tag) => tag.end(),
            SpmlTag::SptDate(tag) => tag.end(),
            SpmlTag::SptDiff(tag) => tag.end(),
            SpmlTag::SptEmail2Img(tag) => tag.end(),
            SpmlTag::SptEncryptemail(tag) => tag.end(),
            SpmlTag::SptEscapeemail(tag) => tag.end(),
            SpmlTag::SptFormsolutions(tag) => tag.end(),
            SpmlTag::SptId2Url(tag) => tag.end(),
            SpmlTag::SptIlink(tag) => tag.end(),
            SpmlTag::SptImageeditor(tag) => tag.end(),
            SpmlTag::SptImp(tag) => tag.end(),
            SpmlTag::SptIterator(tag) => tag.end(),
            SpmlTag::SptLink(tag) => tag.end(),
            SpmlTag::SptNumber(tag) => tag.end(),
            SpmlTag::SptPersonalization(tag) => tag.end(),
            SpmlTag::SptPrehtml(tag) => tag.end(),
            SpmlTag::SptSmarteditor(tag) => tag.end(),
            SpmlTag::SptSpml(tag) => tag.end(),
            SpmlTag::SptText(tag) => tag.end(),
            SpmlTag::SptTextarea(tag) => tag.end(),
            SpmlTag::SptTimestamp(tag) => tag.end(),
            SpmlTag::SptTinymce(tag) => tag.end(),
            SpmlTag::SptUpdown(tag) => tag.end(),
            SpmlTag::SptUpload(tag) => tag.end(),
            SpmlTag::SptWorklist(tag) => tag.end(),
        };
    }
}

// TODO: evaluate
pub(crate) struct DepthCounter {
    value: u8,
}

impl DepthCounter {
    pub(crate) fn new() -> Self {
        return Self { value: 0 };
    }

    pub(crate) fn bump(&mut self) {
        self.value += 1;
    }

    pub(crate) fn get(&self) -> u8 {
        return self.value;
    }
}

macro_rules! tag_struct {
    (#[$definition:expr] $name:ident { $( $param:ident ),* $(,)* }) => {
        #[derive(Clone, Debug, PartialEq, DocumentNode, ParsableTag)]
        #[tag_definition($definition)]
        pub(crate) struct $name {
            pub(crate) open_location: Location,
            $(pub(crate) $param: Option<ParsedAttribute<SpelAttribute>>,)*
            pub(crate) body: Option<TagBody>,
            pub(crate) close_location: Location,
        }

        impl Tag for $name {
            fn start(&self) -> &Location {
                return &self.open_location;
            }

            fn end(&self) -> &Location {
                return &self.close_location;
            }
        }
    };
}

tag_struct!(
    #[TagDefinition::SP_ARGUMENT]
    SpArgument {
        condition_attribute,
        default_attribute,
        expression_attribute,
        locale_attribute,
        name_attribute,
        object_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_ATTRIBUTE]
    SpAttribute {
        dynamics_attribute,
        name_attribute,
        object_attribute,
        text_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_BARCODE]
    SpBarcode {
        height_attribute,
        locale_attribute,
        name_attribute,
        scope_attribute,
        text_attribute,
        type_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_BREAK]
    SpBreak {}
);

tag_struct!(
    #[TagDefinition::SP_CALENDARSHEET]
    SpCalendarsheet {
        action_attribute,
        date_attribute,
        from_attribute,
        mode_attribute,
        name_attribute,
        object_attribute,
        scope_attribute,
        to_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_CHECKBOX]
    SpCheckbox {
        checked_attribute,
        disabled_attribute,
        locale_attribute,
        name_attribute,
        readonly_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_CODE]
    SpCode {}
);

tag_struct!(
    #[TagDefinition::SP_COLLECTION]
    SpCollection {
        action_attribute,
        condition_attribute,
        default_attribute,
        expression_attribute,
        index_attribute,
        locale_attribute,
        name_attribute,
        object_attribute,
        publisher_attribute,
        query_attribute,
        scope_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_CONDITION]
    SpCondition {}
);

tag_struct!(
    #[TagDefinition::SP_DIFF]
    SpDiff {
        from_attribute,
        locale_attribute,
        lookup_attribute,
        name_attribute,
        to_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_ELSE]
    SpElse {}
);

tag_struct!(
    #[TagDefinition::SP_ELSEIF]
    SpElseIf {
        condition_attribute,
        eq_attribute,
        gt_attribute,
        gte_attribute,
        ic_attribute,
        isNull_attribute,
        locale_attribute,
        lt_attribute,
        lte_attribute,
        match_attribute,
        name_attribute,
        neq_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_ERROR]
    SpError { code_attribute }
);

tag_struct!(
    #[TagDefinition::SP_EXPIRE]
    SpExpire { date_attribute }
);

tag_struct!(
    #[TagDefinition::SP_FILTER]
    SpFilter {
        attribute_attribute,
        collection_attribute,
        filter_attribute,
        format_attribute,
        from_attribute,
        ic_attribute,
        invert_attribute,
        locale_attribute,
        mode_attribute,
        name_attribute,
        scope_attribute,
        to_attribute,
        type_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_FOR]
    SpFor {
        condition_attribute,
        from_attribute,
        index_attribute,
        locale_attribute,
        step_attribute,
        to_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_FORM]
    SpForm {
        command_attribute,
        context_attribute,
        enctype_attribute,
        handler_attribute,
        id_attribute,
        locale_attribute,
        method_attribute,
        module_attribute,
        name_attribute,
        nameencoding_attribute,
        template_attribute,
        uri_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_HIDDEN]
    SpHidden {
        fixvalue_attribute,
        locale_attribute,
        name_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_IF]
    SpIf {
        condition_attribute,
        eq_attribute,
        gt_attribute,
        gte_attribute,
        ic_attribute,
        isNull_attribute,
        locale_attribute,
        lt_attribute,
        lte_attribute,
        match_attribute,
        name_attribute,
        neq_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_INCLUDE]
    SpInclude {
        anchor_attribute,
        arguments_attribute,
        context_attribute,
        mode_attribute,
        module_attribute,
        return_attribute,
        template_attribute,
        uri_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_IO]
    SpIo {
        contenttype_attribute,
        type_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_ITERATOR]
    SpIterator {
        collection_attribute,
        item_attribute,
        max_attribute,
        min_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_JSON]
    SpJson {
        indent_attribute,
        locale_attribute,
        name_attribute,
        object_attribute,
        overwrite_attribute,
        scope_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_LINKEDINFORMATION]
    SpLinkedinformation {}
);

tag_struct!(
    #[TagDefinition::SP_LINKTREE]
    SpLinktree {
        attributes_attribute,
        locale_attribute,
        localelink_attribute,
        name_attribute,
        parentlink_attribute,
        rootelement_attribute,
        sortkeys_attribute,
        sortsequences_attribute,
        sorttypes_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_LIVETREE]
    SpLivetree {
        action_attribute,
        leaflink_attribute,
        locale_attribute,
        name_attribute,
        node_attribute,
        parentlink_attribute,
        publisher_attribute,
        rootElement_attribute,
        sortkeys_attribute,
        sortsequences_attribute,
        sorttypes_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_LOG]
    SpLog { level_attribute }
);

tag_struct!(
    #[TagDefinition::SP_LOGIN]
    SpLogin {
        captcharequired_attribute,
        client_attribute,
        locale_attribute,
        login_attribute,
        password_attribute,
        scope_attribute,
        session_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_LOOP]
    SpLoop {
        collection_attribute,
        item_attribute,
        list_attribute,
        locale_attribute,
        separator_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_MAP]
    SpMap {
        action_attribute,
        condition_attribute,
        default_attribute,
        expression_attribute,
        key_attribute,
        locale_attribute,
        name_attribute,
        object_attribute,
        overwrite_attribute,
        scope_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_OPTION]
    SpOption {
        disabled_attribute,
        locale_attribute,
        selected_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_PASSWORD]
    SpPassword {}
);

tag_struct!(
    #[TagDefinition::SP_PRINT]
    SpPrint {
        arg_attribute,
        condition_attribute,
        convert_attribute,
        cryptkey_attribute,
        dateformat_attribute,
        decimalformat_attribute,
        decoding_attribute,
        decrypt_attribute,
        default_attribute,
        encoding_attribute,
        encrypt_attribute,
        expression_attribute,
        locale_attribute,
        name_attribute,
        text_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_QUERYTREE]
    SpQuerytree {}
);

tag_struct!(
    #[TagDefinition::SP_RADIO]
    SpRadio {
        checked_attribute,
        disabled_attribute,
        locale_attribute,
        name_attribute,
        readonly_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_RANGE]
    SpRange {
        collection_attribute,
        name_attribute,
        range_attribute,
        scope_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_RETURN]
    SpReturn {
        condition_attribute,
        default_attribute,
        expression_attribute,
        locale_attribute,
        object_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_SASS]
    SpSass {
        name_attribute,
        options_attribute,
        source_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_SCALEIMAGE]
    SpScaleimage {
        background_attribute,
        height_attribute,
        locale_attribute,
        name_attribute,
        object_attribute,
        options_attribute,
        padding_attribute,
        quality_attribute,
        scalesteps_attribute,
        scope_attribute,
        width_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_SCOPE]
    SpScope { scope_attribute }
);

tag_struct!(
    #[TagDefinition::SP_SEARCH]
    SpSearch {}
);

tag_struct!(
    #[TagDefinition::SP_SELECT]
    SpSelect {
        disabled_attribute,
        locale_attribute,
        multiple_attribute,
        name_attribute,
        type_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_SET]
    SpSet {
        condition_attribute,
        contentType_attribute,
        default_attribute,
        expression_attribute,
        insert_attribute,
        locale_attribute,
        name_attribute,
        object_attribute,
        overwrite_attribute,
        scope_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_SORT]
    SpSort {
        collection_attribute,
        keys_attribute,
        locale_attribute,
        name_attribute,
        scope_attribute,
        sequences_attribute,
        types_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_SUBINFORMATION]
    SpSubinformation {
        name_attribute,
        type_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_TAGBODY]
    SpTagbody {}
);

tag_struct!(
    #[TagDefinition::SP_TEXT]
    SpText {
        disabled_attribute,
        fixvalue_attribute,
        format_attribute,
        inputType_attribute,
        locale_attribute,
        name_attribute,
        readonly_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_TEXTAREA]
    SpTextarea {
        disabled_attribute,
        fixvalue_attribute,
        locale_attribute,
        name_attribute,
        readonly_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_TEXTIMAGE]
    SpTextimage {
        background_attribute,
        fontcolor_attribute,
        fontname_attribute,
        fontsize_attribute,
        fontstyle_attribute,
        gravity_attribute,
        height_attribute,
        locale_attribute,
        name_attribute,
        offset_attribute,
        scope_attribute,
        text_attribute,
        width_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_THROW]
    SpThrow {}
);

tag_struct!(
    #[TagDefinition::SP_TOGGLE]
    SpToggle {
        disabled_attribute,
        fixvalue_attribute,
        locale_attribute,
        name_attribute,
        offValue_attribute,
        onValue_attribute,
        readonly_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_UPLOAD]
    SpUpload {
        locale_attribute,
        name_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_URL]
    SpUrl {
        absolute_attribute,
        command_attribute,
        context_attribute,
        gui_attribute,
        handler_attribute,
        information_attribute,
        locale_attribute,
        module_attribute,
        publisher_attribute,
        template_attribute,
        uri_attribute,
        window_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_WARNING]
    SpWarning { code_attribute }
);

tag_struct!(
    #[TagDefinition::SP_WORKLIST]
    SpWorklist {
        element_attribute,
        name_attribute,
        role_attribute,
        user_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_ZIP]
    SpZip {}
);

tag_struct!(
    #[TagDefinition::SPT_COUNTER]
    SptCounter {
        language_attribute,
        mode_attribute,
        name_attribute,
        varName_attribute,
        varname_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_DATE]
    SptDate {
        disabled_attribute,
        fixvalue_attribute,
        locale_attribute,
        name_attribute,
        nowButton_attribute,
        placeholder_attribute,
        readonly_attribute,
        size_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_DIFF]
    SptDiff {
        from_attribute,
        style_attribute,
        to_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_EMAIL2IMG]
    SptEmail2Img {
        alt_attribute,
        bgcolor_attribute,
        bgcolor2_attribute,
        color_attribute,
        color2_attribute,
        font_attribute,
        font2_attribute,
        fontsize_attribute,
        fontsize2_attribute,
        fontweight_attribute,
        fontweight2_attribute,
        form_attribute,
        linkcolor_attribute,
        name_attribute,
        object_attribute,
        onclick_attribute,
        popupheight_attribute,
        popupwidth_attribute,
        title_attribute,
        urlparam_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_ENCRYPTEMAIL]
    SptEncryptemail {
        form_attribute,
        name_attribute,
        object_attribute,
        popupheight_attribute,
        popupwidth_attribute,
        urlparam_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_ESCAPEEMAIL]
    SptEscapeemail {
        alt_attribute,
        bgcolor_attribute,
        color_attribute,
        font_attribute,
        fontsize_attribute,
        fontweight_attribute,
        form_attribute,
        name_attribute,
        object_attribute,
        onclick_attribute,
        popupheight_attribute,
        popupwidth_attribute,
        title_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_FORMSOLUTIONS]
    SptFormsolutions {
        locale_attribute,
        name_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_ID2URL]
    SptId2Url {
        classname_attribute,
        locale_attribute,
        name_attribute,
        objekt_attribute,
        querystring_attribute,
        url_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_ILINK]
    SptIlink {
        action_attribute,
        information_attribute,
        step_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_IMAGEEDITOR]
    SptImageeditor {
        delete_attribute,
        focalpoint_attribute,
        locale_attribute,
        name_attribute,
        object_attribute,
        width_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_IMP]
    SptImp {
        alt_attribute,
        background_attribute,
        color_attribute,
        excerpt_attribute,
        font_attribute,
        fontcolor_attribute,
        fontname_attribute,
        fontsize_attribute,
        fontweight_attribute,
        format_attribute,
        gravity_attribute,
        height_attribute,
        image_attribute,
        manipulate_attribute,
        offset_attribute,
        padding_attribute,
        paddingcolor_attribute,
        scalesteps_attribute,
        text_attribute,
        transform_attribute,
        urlonly_attribute,
        width_attribute,
    }
);
/* TODO:
        font-size_attribute,
        font-weight_attribute,
        text-transform_attribute,
*/

tag_struct!(
    #[TagDefinition::SPT_ITERATOR]
    SptIterator {
        disabled_attribute,
        invert_attribute,
        item_attribute,
        itemtext_attribute,
        layout_attribute,
        max_attribute,
        min_attribute,
        name_attribute,
        readonly_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_LINK]
    SptLink {
        filter_attribute,
        filterattribute_attribute,
        filteric_attribute,
        filterinvert_attribute,
        filtermode_attribute,
        filterquery_attribute,
        fixvalue_attribute,
        height_attribute,
        hidden_attribute,
        locale_attribute,
        name_attribute,
        pools_attribute,
        previewimage_attribute,
        showtree_attribute,
        size_attribute,
        type_attribute,
        value_attribute,
        width_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_NUMBER]
    SptNumber {
        align_attribute,
        disabled_attribute,
        fixvalue_attribute,
        locale_attribute,
        name_attribute,
        readonly_attribute,
        size_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_PERSONALIZATION]
    SptPersonalization {
        information_attribute,
        mode_attribute,
        name_attribute,
        publisher_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_PREHTML]
    SptPrehtml {
        name_attribute,
        object_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_SMARTEDITOR]
    SptSmarteditor {
        cols_attribute,
        hide_attribute,
        name_attribute,
        options_attribute,
        rows_attribute,
        textlabel_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_SPML]
    SptSpml { api_attribute }
);

tag_struct!(
    #[TagDefinition::SPT_TEXT]
    SptText {
        disabled_attribute,
        editablePlaceholder_attribute,
        fixvalue_attribute,
        format_attribute,
        hyphenEditor_attribute,
        inputType_attribute,
        locale_attribute,
        name_attribute,
        readonly_attribute,
        size_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_TEXTAREA]
    SptTextarea {
        disabled_attribute,
        editablePlaceholder_attribute,
        fixvalue_attribute,
        format_attribute,
        hyphenEditor_attribute,
        inputType_attribute,
        locale_attribute,
        name_attribute,
        readonly_attribute,
        size_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_TIMESTAMP]
    SptTimestamp { connect_attribute }
);

tag_struct!(
    #[TagDefinition::SPT_TINYMCE]
    SptTinymce {
        cols_attribute,
        config_attribute,
        configextension_attribute,
        configvalues_attribute,
        disabled_attribute,
        fixvalue_attribute,
        name_attribute,
        pools_attribute,
        readonly_attribute,
        rows_attribute,
        theme_attribute,
        toggle_attribute,
        type_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_UPDOWN]
    SptUpdown {
        from_attribute,
        locale_attribute,
        name_attribute,
        to_attribute,
        value_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_UPLOAD]
    SptUpload {
        locale_attribute,
        name_attribute,
        previewimage_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SPT_WORKLIST]
    SptWorklist {
        command_attribute,
        informationID_attribute,
        poolID_attribute,
        worklistID_attribute,
    }
);

pub(crate) trait Tag {
    fn start(&self) -> &Location;

    fn end(&self) -> &Location;
}

pub(crate) trait Attribute {
    fn start(&self) -> &Location;

    fn end(&self) -> &Location;
}

pub(crate) trait AttributeValue {
    fn opening_quote_location(&self) -> &Location;

    fn closing_quote_location(&self) -> &Location;

    fn is_inside(&self, position: &Position) -> bool {
        let opening_line = self
            .opening_quote_location()
            .line
            .cmp(&(position.line as usize));
        let opening_char = self
            .opening_quote_location()
            .char
            .cmp(&(position.character as usize));
        match (opening_line, opening_char) {
            (Ordering::Less, _) | (Ordering::Equal, Ordering::Less) => (),
            _ => return false,
        }
        let closing_line = self
            .closing_quote_location()
            .line
            .cmp(&(position.line as usize));
        let closing_char = self
            .closing_quote_location()
            .char
            .cmp(&(position.character as usize));
        return match (closing_line, closing_char) {
            (Ordering::Greater, _)
            | (Ordering::Equal, Ordering::Greater)
            | (Ordering::Equal, Ordering::Equal) => true,
            _ => false,
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlainAttribute {
    pub(crate) key_location: Location,
    pub(crate) value: Option<PlainAttributeValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlainAttributeValue {
    pub(crate) equals_location: Location,
    pub(crate) opening_quote_location: Location,
    pub(crate) value: String,
    pub(crate) closing_quote_location: Location,
}

// TODO:
// PlainAttribute should be HtmlAttribute
// PlainAttribute.value should not be an Option

// #[derive(Clone, Debug, PartialEq)]
// pub(crate) struct HtmlAttributeValue {
//     pub(crate) equals_location: Location,
//     pub(crate) opening_quote_location: Location,
//     pub(crate) fragments: HtmlAttributeValueFragments,
//     pub(crate) closing_quote_location: Location,
// }

// #[derive(Clone, Debug, PartialEq)]
// pub(crate) struct HtmlAttributeValueFragments(Vec<HtmlAttributeValueFragment>);

// #[derive(Clone, Debug, PartialEq)]
// pub(crate) enum HtmlAttributeValueFragment {
//     Plain(String),
//     Tag(Tag),
// }

impl AttributeValue for PlainAttributeValue {
    fn opening_quote_location(&self) -> &Location {
        return &self.opening_quote_location;
    }

    fn closing_quote_location(&self) -> &Location {
        return &self.closing_quote_location;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParsedAttribute<A: Attribute> {
    Valid(A),
    Erroneous(A, Vec<AttributeError>),
    Unparsable(String, Location),
}

impl<R: Attribute> ParsedAttribute<R> {
    fn start(&self) -> &Location {
        return match &self {
            ParsedAttribute::Valid(a) => a.start(),
            ParsedAttribute::Erroneous(a, _) => a.start(),
            ParsedAttribute::Unparsable(_, location) => location,
        };
    }

    fn end(&self) -> &Location {
        return match &self {
            ParsedAttribute::Valid(attribute) => attribute.end(),
            ParsedAttribute::Erroneous(attribute, _) => attribute.end(),
            ParsedAttribute::Unparsable(_, location) => location,
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AttributeError {
    Superfluous(String, Location),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParsedTag<A: Tag> {
    Valid(A),
    Erroneous(A, Vec<TagError>),
    Unparsable(String, Location),
}

impl<R: Tag> ParsedTag<R> {
    fn map<T>(&self, function: fn(&R) -> T) -> ParsedTag<T>
    where
        T: Tag,
    {
        return match &self {
            ParsedTag::Valid(tag) => ParsedTag::Valid(function(tag)),
            ParsedTag::Erroneous(tag, errors) => {
                ParsedTag::Erroneous(function(tag), errors.to_vec())
            }
            ParsedTag::Unparsable(text, location) => {
                ParsedTag::Unparsable(text.clone(), location.clone())
            }
        };
    }
}

impl<T: Tag> DocumentNode for ParsedTag<T> {
    fn range(&self) -> Range {
        let (start, end) = match self {
            ParsedTag::Valid(tag) => (tag.start(), tag.end()),
            ParsedTag::Erroneous(tag, _) => (tag.start(), tag.end()),
            ParsedTag::Unparsable(_, location) => return location.range(),
        };
        return Range {
            start: Position {
                line: start.line as u32,
                character: start.char as u32,
            },
            end: Position {
                line: end.line as u32,
                character: (end.char + end.length) as u32,
            },
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagError {
    Superfluous(String, Location),
    Missing(String, Location),
}

impl Attribute for PlainAttribute {
    fn start(&self) -> &Location {
        return &self.key_location;
    }

    fn end(&self) -> &Location {
        return match &self.value {
            Some(value) => &value.closing_quote_location,
            None => &self.key_location,
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpelAttribute {
    pub(crate) key_location: Location,
    pub(crate) value: SpelAttributeValue,
}

impl Attribute for SpelAttribute {
    fn start(&self) -> &Location {
        return &self.key_location;
    }

    fn end(&self) -> &Location {
        return &self.value.closing_quote_location;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpelAttributeValue {
    pub(crate) equals_location: Location,
    pub(crate) opening_quote_location: Location,
    pub(crate) spel: SpelAst,
    pub(crate) closing_quote_location: Location,
}

impl AttributeValue for SpelAttributeValue {
    fn opening_quote_location(&self) -> &Location {
        return &self.opening_quote_location;
    }

    fn closing_quote_location(&self) -> &Location {
        return &self.closing_quote_location;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Location {
    pub(crate) char: usize,
    pub(crate) line: usize,
    pub(crate) length: usize,
}

impl Location {
    pub(crate) fn new(char: usize, line: usize, length: usize) -> Self {
        return Location { char, line, length };
    }

    pub(crate) fn contains(&self, position: &Position) -> bool {
        return self.line == position.line as usize
            && self.char <= position.character as usize
            && self.char + self.length > position.character as usize;
    }

    pub(crate) fn range(&self) -> Range {
        return Range {
            start: Position {
                line: self.line as u32,
                character: self.char as u32,
            },
            end: Position {
                line: self.line as u32,
                character: (self.char + self.length) as u32,
            },
        };
    }
}

pub(crate) struct TreeParser<'tree> {
    cursor: tree_sitter::TreeCursor<'tree>,
    text_bytes: &'tree [u8],
}

#[derive(Clone, Debug, PartialEq)]
enum NodeMovement {
    NextSibling,
    FirstChild,
    Current,
}

#[derive(Clone, Debug, PartialEq)]
enum NodeMovingResult<'a> {
    NonExistent,
    Missing(tree_sitter::Node<'a>),
    Erroneous(tree_sitter::Node<'a>),
    Superfluous(tree_sitter::Node<'a>),
    Ok(tree_sitter::Node<'a>),
}

struct AttributeParser<'a, 'b> {
    tree_parser: &'a mut TreeParser<'b>,
    parent_node: tree_sitter::Node<'b>,
    errors: Option<Vec<AttributeError>>,
    depth: u8,
}

enum IntermediateAttributeParsingResult<R> {
    Failed(String, Location),
    Partial(R),
}

impl<'a, 'b> AttributeParser<'a, 'b> {
    fn plain(
        tree_parser: &'a mut TreeParser<'b>,
    ) -> Result<(String, ParsedAttribute<PlainAttribute>)> {
        let mut parser = AttributeParser::new(tree_parser);
        let result = parser.parse_plain();
        parser.walk_back();
        return result;
    }

    // fn html(
    //     tree_parser: &'a mut TreeParser<'b>,
    // ) -> Result<(String, ParsedAttribute<HtmlAttribute>)> {
    //     let mut parser = AttributeParser::new(tree_parser);
    //     let result = parser.parse_plain();
    //     parser.walk_back();
    //     return result;
    // }

    fn spel(
        tree_parser: &'a mut TreeParser<'b>,
        r#type: &TagAttributeType,
    ) -> Result<(String, ParsedAttribute<SpelAttribute>)> {
        let mut parser = AttributeParser::new(tree_parser);
        let result = parser.parse_spel(r#type);
        parser.walk_back();
        return result;
    }

    fn new(tree_parser: &'a mut TreeParser<'b>) -> Self {
        let parent_node = tree_parser.cursor.node();
        return AttributeParser {
            tree_parser,
            parent_node,
            errors: None,
            depth: 0,
        };
    }

    fn walk_back(&mut self) {
        for _ in 0..self.depth {
            self.tree_parser.cursor.goto_parent();
        }
    }

    fn parse_plain(&mut self) -> Result<(String, ParsedAttribute<PlainAttribute>)> {
        let key_node = match self.parse_key()? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok((
                    // TODO: this is ass
                    "".to_string(),
                    ParsedAttribute::Unparsable(message, location),
                ));
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let key_location = node_location(key_node);
        let key = self.tree_parser.node_text(&key_node)?.to_string();
        let equals_location = match self.parse_equals(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok((key, ParsedAttribute::Unparsable(message, location)))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let value = match equals_location {
            Some(equals_location) => {
                if let IntermediateAttributeParsingResult::Failed(message, location) =
                    self.parse_string(&key_node)?
                {
                    return Ok((key, ParsedAttribute::Unparsable(message, location)));
                }
                let opening_quote_location = match self.parse_opening_quote(&key_node)? {
                    IntermediateAttributeParsingResult::Failed(message, location) => {
                        return Ok((key, ParsedAttribute::Unparsable(message, location)))
                    }
                    IntermediateAttributeParsingResult::Partial(e) => e,
                };
                let (value, movement) = match self.parse_string_content(&key_node)? {
                    IntermediateAttributeParsingResult::Failed(message, location) => {
                        return Ok((key, ParsedAttribute::Unparsable(message, location)))
                    }
                    IntermediateAttributeParsingResult::Partial(e) => e,
                };
                let closing_quote_location = match self.parse_closing_quote(&key_node, movement)? {
                    IntermediateAttributeParsingResult::Failed(message, location) => {
                        return Ok((key, ParsedAttribute::Unparsable(message, location)))
                    }
                    IntermediateAttributeParsingResult::Partial(e) => e,
                };
                Some(PlainAttributeValue {
                    equals_location,
                    opening_quote_location,
                    value,
                    closing_quote_location,
                })
            }
            None => None,
        };
        let attribute = PlainAttribute {
            key_location,
            value,
        };
        let parsed = match &self.errors {
            Some(errors) => ParsedAttribute::Erroneous(attribute, errors.to_vec()),
            None => ParsedAttribute::Valid(attribute),
        };
        return Ok((key, parsed));
    }

    fn parse_spel(
        &mut self,
        r#type: &TagAttributeType,
    ) -> Result<(String, ParsedAttribute<SpelAttribute>)> {
        let key_node = match self.parse_key()? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok((
                    // TODO: this is ass
                    "".to_string(),
                    ParsedAttribute::Unparsable(message, location),
                ));
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let key_location = node_location(key_node);
        let key = self.tree_parser.node_text(&key_node)?.to_string();
        let equals_location = match self.parse_equals(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok((key, ParsedAttribute::Unparsable(message, location)))
            }
            IntermediateAttributeParsingResult::Partial(Some(e)) => e,
            IntermediateAttributeParsingResult::Partial(None) => {
                return Ok((
                    key,
                    ParsedAttribute::Unparsable(
                        "missing \"=\"".to_string(),
                        node_location(self.parent_node),
                    ),
                ))
            }
        };
        if let IntermediateAttributeParsingResult::Failed(message, location) =
            self.parse_string(&key_node)?
        {
            return Ok((key, ParsedAttribute::Unparsable(message, location)));
        }
        let opening_quote_location = match self.parse_opening_quote(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok((key, ParsedAttribute::Unparsable(message, location)))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let (spel, movement) = match self.parse_spel_content(&key_node, r#type)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok((key, ParsedAttribute::Unparsable(message, location)))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let closing_quote_location = match self.parse_closing_quote(&key_node, movement)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok((key, ParsedAttribute::Unparsable(message, location)))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let attribute = SpelAttribute {
            key_location,
            value: SpelAttributeValue {
                equals_location,
                opening_quote_location,
                spel,
                closing_quote_location,
            },
        };
        let parsed = match &self.errors {
            Some(errors) => ParsedAttribute::Erroneous(attribute, errors.to_vec()),
            None => ParsedAttribute::Valid(attribute),
        };
        return Ok((key, parsed));
    }

    fn parse_key(&mut self) -> Result<IntermediateAttributeParsingResult<tree_sitter::Node<'a>>> {
        let mut movement = &NodeMovement::FirstChild;
        loop {
            match self.goto(movement) {
                // probably cannot happen...
                NodeMovingResult::NonExistent | NodeMovingResult::Missing(_) => {
                    return Ok(IntermediateAttributeParsingResult::Failed(
                        "missing attribute".to_string(),
                        node_location(self.parent_node),
                    ));
                }
                NodeMovingResult::Erroneous(node) => {
                    return Ok(IntermediateAttributeParsingResult::Failed(
                        format!(
                            "invalid attribute \"{}\"",
                            self.tree_parser.node_text(&node)?
                        ),
                        node_location(node),
                    ));
                }
                NodeMovingResult::Superfluous(node) => {
                    self.add_error(AttributeError::Superfluous(
                        self.tree_parser.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    movement = &NodeMovement::NextSibling;
                    continue;
                }
                NodeMovingResult::Ok(node) => {
                    return Ok(IntermediateAttributeParsingResult::Partial(node))
                }
            };
        }
    }

    fn parse_equals(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
    ) -> Result<IntermediateAttributeParsingResult<Option<Location>>> {
        loop {
            return Ok(match self.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Partial(None),
                NodeMovingResult::Missing(_) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "missing \"=\" after attribute name \"{}\"",
                        self.tree_parser.node_text(key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Erroneous(node) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "expected \"=\", found \"{}\"",
                        self.tree_parser.node_text(&node)?
                    ),
                    node_location(node),
                ),
                NodeMovingResult::Superfluous(node) => {
                    self.add_error(AttributeError::Superfluous(
                        self.tree_parser.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(node) => {
                    IntermediateAttributeParsingResult::Partial(Some(node_location(node)))
                }
            });
        }
    }

    fn parse_string(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
    ) -> Result<IntermediateAttributeParsingResult<()>> {
        loop {
            return Ok(match self.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "missing attribute value for \"{}\"",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                // TODO: Missing may be recoverable
                NodeMovingResult::Missing(_) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "missing attribute value for \"{}\"",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Erroneous(node) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "expected attribute value, found \"{}\"",
                        self.tree_parser.node_text(&node)?
                    ),
                    node_location(node),
                ),
                NodeMovingResult::Superfluous(node) => {
                    self.add_error(AttributeError::Superfluous(
                        self.tree_parser.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(_) => IntermediateAttributeParsingResult::Partial(()),
            });
        }
    }

    fn parse_opening_quote(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
    ) -> Result<IntermediateAttributeParsingResult<Location>> {
        loop {
            return Ok(match self.goto(&NodeMovement::FirstChild) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "attribute \"{}\" is missing a value",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Missing(_) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "missing \"\"\" after attribute name \"{}=\"",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Erroneous(node) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "expected \"\"\", found \"{}\"",
                        self.tree_parser.node_text(&node)?
                    ),
                    node_location(node),
                ),
                NodeMovingResult::Superfluous(node) => {
                    self.add_error(AttributeError::Superfluous(
                        self.tree_parser.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(node) => {
                    IntermediateAttributeParsingResult::Partial(node_location(node))
                }
            });
        }
    }

    fn parse_string_content(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
    ) -> Result<IntermediateAttributeParsingResult<(String, NodeMovement)>> {
        loop {
            return Ok(match self.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" attribute value string is unclosed",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Missing(_) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" attribute value string is unclosed",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Erroneous(node) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "expected \"\"\", found \"{}\"",
                        self.tree_parser.node_text(&node)?
                    ),
                    node_location(node),
                ),
                NodeMovingResult::Superfluous(node) => {
                    self.add_error(AttributeError::Superfluous(
                        self.tree_parser.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(node) if node.kind() == "\"" => {
                    IntermediateAttributeParsingResult::Partial((
                        "".to_string(),
                        NodeMovement::Current,
                    ))
                }
                NodeMovingResult::Ok(node) => IntermediateAttributeParsingResult::Partial((
                    self.tree_parser.node_text(&node)?.to_string(),
                    NodeMovement::NextSibling,
                )),
            });
        }
    }

    fn parse_spel_content(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
        r#type: &TagAttributeType,
    ) -> Result<IntermediateAttributeParsingResult<(SpelAst, NodeMovement)>> {
        let (text, movement) = match self.parse_string_content(key_node)? {
            IntermediateAttributeParsingResult::Partial(e) => e,
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(IntermediateAttributeParsingResult::Failed(
                    message, location,
                ))
            }
        };
        let parser = &mut spel::parser::Parser::new(&text);
        let spel = match r#type {
            TagAttributeType::Comparable => match parser.parse_comparable() {
                Ok(result) => SpelAst::Comparable(SpelResult::Valid(result)),
                // workaround as comparables as attribute values do accept strings (without quotes)
                // but comparables in actual comparissons do not.
                Err(err) => match spel::parser::Parser::new(&text).parse_text() {
                    Ok(result) => SpelAst::String(SpelResult::Valid(result)),
                    Err(_) => SpelAst::Comparable(SpelResult::Invalid(err)),
                },
            },
            TagAttributeType::Condition => SpelAst::Condition(match parser.parse_condition_ast() {
                Ok(result) => SpelResult::Valid(result.root),
                Err(err) => SpelResult::Invalid(err),
            }),
            TagAttributeType::Expression => {
                SpelAst::Expression(match parser.parse_expression_ast() {
                    Ok(result) => SpelResult::Valid(result.root),
                    Err(err) => SpelResult::Invalid(err),
                })
            }
            TagAttributeType::Identifier => SpelAst::Identifier(match parser.parse_identifier() {
                Ok(result) => SpelResult::Valid(result),
                Err(err) => SpelResult::Invalid(err),
            }),
            TagAttributeType::Module => SpelAst::String(match parser.parse_text() {
                Ok(result) => SpelResult::Valid(result),
                Err(err) => SpelResult::Invalid(err),
            }),
            TagAttributeType::Object => SpelAst::Object(match parser.parse_object_ast() {
                Ok(result) => SpelResult::Valid(result.root),
                Err(err) => SpelResult::Invalid(err),
            }),
            TagAttributeType::Regex => SpelAst::Regex(match parser.parse_regex() {
                Ok(result) => SpelResult::Valid(result),
                Err(err) => SpelResult::Invalid(err),
            }),
            TagAttributeType::String => SpelAst::String(match parser.parse_text() {
                Ok(result) => SpelResult::Valid(result),
                Err(err) => SpelResult::Invalid(err),
            }),
            TagAttributeType::Query => SpelAst::Query(match parser.parse_query() {
                Ok(result) => SpelResult::Valid(result),
                Err(err) => SpelResult::Invalid(err),
            }),
            TagAttributeType::Uri { .. } => SpelAst::Uri(match parser.parse_uri() {
                Ok(result) => SpelResult::Valid(result),
                Err(err) => SpelResult::Invalid(err),
            }),
        };
        return Ok(IntermediateAttributeParsingResult::Partial((
            spel, movement,
        )));
    }

    fn parse_closing_quote(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
        movement: NodeMovement,
    ) -> Result<IntermediateAttributeParsingResult<Location>> {
        loop {
            return Ok(match self.goto(&movement) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" attribute value string is unclosed",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Missing(_) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" attribute value string is unclosed",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Erroneous(node) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "expected \"\"\", found \"{}\"",
                        self.tree_parser.node_text(&node)?
                    ),
                    node_location(node),
                ),
                NodeMovingResult::Superfluous(node) => {
                    self.add_error(AttributeError::Superfluous(
                        self.tree_parser.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(node) => {
                    IntermediateAttributeParsingResult::Partial(node_location(node))
                }
            });
        }
    }

    // TODO: DepthCounter?
    fn goto(&mut self, movement: &NodeMovement) -> NodeMovingResult<'a> {
        let result = self.tree_parser.goto(movement);
        match (movement, &result) {
            (_, NodeMovingResult::NonExistent) => (),
            (NodeMovement::FirstChild, _) => {
                self.depth += 1;
            }
            _ => (),
        }
        return result;
    }

    fn add_error(&mut self, error: AttributeError) {
        match &mut self.errors {
            None => self.errors = Some(vec![error]),
            Some(errors) => errors.push(error),
        }
    }
}

impl<'a> TreeParser<'a> {
    pub(crate) fn new<'tree>(
        cursor: tree_sitter::TreeCursor<'tree>,
        text: &'tree String,
    ) -> TreeParser<'tree> {
        return TreeParser {
            cursor,
            text_bytes: text.as_bytes(),
        };
    }

    fn goto(&mut self, movement: &NodeMovement) -> NodeMovingResult<'a> {
        let moved = match movement {
            NodeMovement::NextSibling => self.cursor.goto_next_sibling(),
            NodeMovement::FirstChild => self.cursor.goto_first_child(),
            NodeMovement::Current => true,
        };
        if !moved {
            return NodeMovingResult::NonExistent;
        }
        let node = self.cursor.node();
        if !node.is_missing() {
            if !node.is_extra() {
                if !node.is_error() {
                    return NodeMovingResult::Ok(node);
                }
                return NodeMovingResult::Erroneous(node);
            }
            return NodeMovingResult::Superfluous(node);
        }
        return NodeMovingResult::Missing(node);
    }

    fn node_text(&self, node: &tree_sitter::Node<'_>) -> Result<&str, Utf8Error> {
        return node.utf8_text(self.text_bytes);
    }

    pub(crate) fn parse_header(&mut self) -> Result<Header> {
        let root = self.cursor.node();
        if root.kind() != "document" {
            return Err(anyhow::anyhow!(
                "missplaced cursor. the header should be the first thing that a TreeParser parses"
            ));
        }
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("document is empty"));
        }
        let mut java_headers = Vec::new();
        let mut taglib_imports = Vec::new();
        loop {
            let header_node = self.cursor.node();
            match header_node.kind() {
                "page_header" => java_headers.push(self.parse_page_header()?),
                "taglib_header" => taglib_imports.push(self.parse_taglib_header()?),
                "comment" | "xml_comment" => (),
                _ => break,
            }
            if !self.cursor.goto_next_sibling() {
                // document contains nothing but the header
                break;
            }
        }
        return Ok(Header {
            java_headers,
            taglib_imports,
        });
    }

    fn parse_page_header(&mut self) -> Result<ParsedNode<PageHeader, IncompletePageHeader>> {
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("java header is empty"));
        }
        let node = self.cursor.node();
        let open_bracket = match (node.is_missing(), node.is_error()) {
            (false, false) => ParsedLocation::Valid(node_location(node)),
            (_, true) => ParsedLocation::Erroneous(node_location(node)),
            (true, _) => ParsedLocation::Missing,
        };

        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let node = self.cursor.node();
        let page = match (node.is_missing(), node.is_error()) {
            (false, false) => ParsedLocation::Valid(node_location(node)),
            (_, true) => ParsedLocation::Erroneous(node_location(node)),
            (true, _) => ParsedLocation::Missing,
        };

        let mut content_type = None;
        let mut language = None;
        let mut page_encoding = None;
        let mut imports = Vec::new();
        let mut node;
        loop {
            if !self.cursor.goto_next_sibling() {
                return Err(anyhow::anyhow!("java header is unclosed"));
            }
            node = self.cursor.node();
            match node.kind() {
                "import_attribute" => imports.push(self.parse_plain_attribute()?.1),
                "contentType_attribute" => content_type = Some(self.parse_plain_attribute()?.1),
                "language_attribute" => language = Some(self.parse_plain_attribute()?.1),
                "pageEncoding_attribute" => page_encoding = Some(self.parse_plain_attribute()?.1),
                "header_close" => break,
                kind => return Err(anyhow::anyhow!("unexpected {}", kind)),
            }
        }
        let close_bracket = match (node.is_missing(), node.is_error()) {
            (false, false) => ParsedLocation::Valid(node_location(node)),
            (_, true) => ParsedLocation::Erroneous(node_location(node)),
            (true, _) => ParsedLocation::Missing,
        };
        self.cursor.goto_parent();
        return Ok(match (open_bracket, page, close_bracket) {
            (
                ParsedLocation::Valid(open_bracket),
                ParsedLocation::Valid(page),
                ParsedLocation::Valid(close_bracket),
            ) => ParsedNode::Valid(PageHeader {
                open_bracket,
                page,
                language,
                page_encoding,
                content_type,
                imports,
                close_bracket,
            }),
            (open_bracket, page, close_bracket) => ParsedNode::Incomplete(IncompletePageHeader {
                open_bracket,
                page,
                language,
                page_encoding,
                content_type,
                imports,
                close_bracket,
            }),
        });
    }

    fn parse_taglib_header(&mut self) -> Result<ParsedNode<TagLibImport, IncompleteTagLibImport>> {
        let mut open_bracket = ParsedLocation::Missing;
        let mut taglib = ParsedLocation::Missing;
        let mut origin = None;
        let mut prefix = None;
        let mut close_bracket = ParsedLocation::Missing;
        let mut errors = Vec::new();
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("java header is empty"));
        }
        let node = self.cursor.node();
        match node.kind() {
            "ERROR" => {
                errors.push(self.parse_error()?);
                if self.cursor.goto_next_sibling() {
                    // TODO
                }
            }
            "header_open" => {
                if !node.is_missing() {
                    open_bracket = ParsedLocation::Valid(node_location(node));
                }
                if self.cursor.goto_next_sibling() {
                    let node = self.cursor.node();
                    match node.kind() {
                        "ERROR" => {
                            errors.push(self.parse_error()?);
                            if self.cursor.goto_next_sibling() {
                                // TODO
                            }
                        }
                        "taglib" => {
                            if !node.is_missing() {
                                taglib = ParsedLocation::Valid(node_location(node));
                            }
                            while self.cursor.goto_next_sibling() {
                                let node = self.cursor.node();
                                match node.kind() {
                                    "ERROR" => {
                                        errors.push(self.parse_error()?);
                                    }
                                    "uri_attribute" => {
                                        origin =
                                            Some(TagLibOrigin::Uri(self.parse_plain_attribute()?.1))
                                    }
                                    "tagdir_attribute" => {
                                        origin = Some(TagLibOrigin::TagDir(
                                            self.parse_plain_attribute()?.1,
                                        ))
                                    }
                                    "prefix_attribute" => {
                                        prefix = Some(self.parse_plain_attribute()?.1)
                                    }
                                    "header_close" => {
                                        if !node.is_missing() {
                                            close_bracket =
                                                ParsedLocation::Valid(node_location(node));
                                        }
                                        break;
                                    },
                                    kind => return Err(anyhow::anyhow!(
                                        "expected 'uri' attribute, 'tagdir' attribute, 'prefix' attribute or '%>', got '{}'",
                                        kind
                                    )),
                                };
                            }
                        }
                        kind => return Err(anyhow::anyhow!("expected 'taglib', got '{}'", kind)),
                    };
                }
            }
            kind => return Err(anyhow::anyhow!("expected taglib header, got '{}'", kind)),
        }
        self.cursor.goto_parent();
        return Ok(
            match (
                open_bracket,
                taglib,
                origin,
                prefix,
                close_bracket,
                errors.len(),
            ) {
                (
                    ParsedLocation::Valid(open_bracket),
                    ParsedLocation::Valid(taglib),
                    Some(origin),
                    Some(prefix),
                    ParsedLocation::Valid(close_bracket),
                    0,
                ) => ParsedNode::Valid(TagLibImport {
                    open_bracket,
                    taglib,
                    origin,
                    prefix,
                    close_bracket,
                }),
                (open_bracket, taglib, origin, prefix, close_bracket, _) => {
                    ParsedNode::Incomplete(IncompleteTagLibImport {
                        open_bracket,
                        taglib,
                        origin,
                        prefix,
                        close_bracket,
                        errors,
                    })
                }
            },
        );
    }

    fn parse_tags(&mut self) -> Result<Vec<Node>> {
        let mut tags = Vec::new();
        loop {
            let node = self.cursor.node();
            match node.kind() {
                "comment" | "xml_comment" | "html_doctype" | "java_tag" => (),
                "text" | "xml_entity" => {
                    tags.push(self.parse_text().map(Node::Text)?);
                    continue;
                }
                "ERROR" => tags.push(self.parse_error().map(Node::Error)?),
                "html_tag" | "html_option_tag" | "html_void_tag" | "script_tag" | "style_tag" => {
                    tags.push(self.parse_html().map(Node::Html)?);
                }
                "attribute_tag" => tags.push(
                    SpAttribute::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpAttribute(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "argument_tag" => tags.push(
                    SpArgument::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpArgument(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "barcode_tag" => tags.push(
                    SpBarcode::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpBarcode(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "break_tag" => tags.push(
                    SpBreak::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpBreak(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "calendarsheet_tag" => tags.push(
                    SpCalendarsheet::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpCalendarsheet(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "checkbox_tag" => tags.push(
                    SpCheckbox::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpCheckbox(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "code_tag" => tags.push(
                    SpCode::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpCode(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "collection_tag" => tags.push(
                    SpCollection::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpCollection(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "condition_tag" => tags.push(
                    SpCondition::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpCondition(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "diff_tag" => tags.push(
                    SpDiff::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpDiff(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "else_tag" => tags.push(
                    SpElse::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpElse(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "elseIf_tag" => tags.push(
                    SpElseIf::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpElseIf(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "error_tag" => tags.push(
                    SpError::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpError(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "expire_tag" => tags.push(
                    SpExpire::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpExpire(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "filter_tag" => tags.push(
                    SpFilter::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpFilter(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "for_tag" => tags.push(
                    SpFor::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpFor(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "form_tag" => tags.push(
                    SpForm::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpForm(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "hidden_tag" => tags.push(
                    SpHidden::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpHidden(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "if_tag" => tags.push(
                    SpIf::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpIf(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "include_tag" => tags.push(
                    SpInclude::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpInclude(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "io_tag" => tags.push(
                    SpIo::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpIo(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "iterator_tag" => tags.push(
                    SpIterator::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpIterator(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "json_tag" => tags.push(
                    SpJson::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpJson(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "linkedinformation_tag" => tags.push(
                    SpLinkedinformation::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpLinkedinformation(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "linktree_tag" => tags.push(
                    SpLinktree::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpLinktree(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "livetree_tag" => tags.push(
                    SpLivetree::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpLivetree(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "log_tag" => tags.push(
                    SpLog::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpLog(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "login_tag" => tags.push(
                    SpLogin::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpLogin(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "loop_tag" => tags.push(
                    SpLoop::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpLoop(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "map_tag" => tags.push(
                    SpMap::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpMap(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "option_tag" => tags.push(
                    SpOption::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpOption(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "password_tag" => tags.push(
                    SpPassword::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpPassword(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "print_tag" => tags.push(
                    SpPrint::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpPrint(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "querytree_tag" => tags.push(
                    SpQuerytree::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpQuerytree(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "radio_tag" => tags.push(
                    SpRadio::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpRadio(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "range_tag" => tags.push(
                    SpRange::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpRange(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "return_tag" => tags.push(
                    SpReturn::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpReturn(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "sass_tag" => tags.push(
                    SpSass::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpSass(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "scaleimage_tag" => tags.push(
                    SpScaleimage::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpScaleimage(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "scope_tag" => tags.push(
                    SpScope::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpScope(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "search_tag" => tags.push(
                    SpSearch::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpSearch(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "select_tag" => tags.push(
                    SpSelect::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpSelect(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "set_tag" => tags.push(
                    SpSet::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpSet(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "sort_tag" => tags.push(
                    SpSort::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpSort(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "subinformation_tag" => tags.push(
                    SpSubinformation::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpSubinformation(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "tagbody_tag" => tags.push(
                    SpTagbody::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpTagbody(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "text_tag" => tags.push(
                    SpText::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpText(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "textarea_tag" => tags.push(
                    SpTextarea::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpTextarea(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "textimage_tag" => tags.push(
                    SpTextimage::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpTextimage(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "throw_tag" => tags.push(
                    SpThrow::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpThrow(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "toggle_tag" => tags.push(
                    SpToggle::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpToggle(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "upload_tag" => tags.push(
                    SpUpload::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpUpload(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "url_tag" => tags.push(
                    SpUrl::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpUrl(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "warning_tag" => tags.push(
                    SpWarning::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpWarning(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "worklist_tag" => tags.push(
                    SpWorklist::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpWorklist(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "zip_tag" => tags.push(
                    SpZip::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SpZip(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_counter_tag" => tags.push(
                    SptCounter::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptCounter(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_date_tag" => tags.push(
                    SptDate::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptDate(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_diff_tag" => tags.push(
                    SptDiff::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptDiff(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_email2img_tag" => tags.push(
                    SptEmail2Img::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptEmail2Img(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_encryptemail_tag" => tags.push(
                    SptEncryptemail::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptEncryptemail(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_escapeemail_tag" => tags.push(
                    SptEscapeemail::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptEscapeemail(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_formsolutions_tag" => tags.push(
                    SptFormsolutions::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptFormsolutions(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_id2url_tag" => tags.push(
                    SptId2Url::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptId2Url(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_ilink_tag" => tags.push(
                    SptIlink::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptIlink(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_imageeditor_tag" => tags.push(
                    SptImageeditor::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptImageeditor(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_imp_tag" => tags.push(
                    SptImp::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptImp(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_iterator_tag" => tags.push(
                    SptIterator::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptIterator(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_link_tag" => tags.push(
                    SptLink::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptLink(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_number_tag" => tags.push(
                    SptNumber::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptNumber(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_personalization_tag" => tags.push(
                    SptPersonalization::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptPersonalization(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_prehtml_tag" => tags.push(
                    SptPrehtml::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptPrehtml(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_smarteditor_tag" => tags.push(
                    SptSmarteditor::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptSmarteditor(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_spml_tag" => tags.push(
                    SptSpml::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptSpml(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_text_tag" => tags.push(
                    SptText::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptText(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_textarea_tag" => tags.push(
                    SptTextarea::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptTextarea(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_timestamp_tag" => tags.push(
                    SptTimestamp::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptTimestamp(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_tinymce_tag" => tags.push(
                    SptTinymce::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptTinymce(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_updown_tag" => tags.push(
                    SptUpdown::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptUpdown(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_upload_tag" => tags.push(
                    SptUpload::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptUpload(tag.clone())))
                        .map(Node::Tag)?,
                ),
                "spt_worklist_tag" => tags.push(
                    SptWorklist::parse(self)
                        .map(|parsed| parsed.map(|tag| SpmlTag::SptWorklist(tag.clone())))
                        .map(Node::Tag)?,
                ),
                kind if kind.ends_with("_tag_close") => break,
                kind => log::debug!("encountered unexpected tree sitter node {}", kind),
            };
            if !self.cursor.goto_next_sibling() {
                break;
            }
        }
        return Ok(tags);
    }

    fn parse_html(&mut self) -> Result<HtmlNode> {
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("html tag is empty"));
        }
        let node = self.cursor.node();
        let name = node.utf8_text(self.text_bytes)?.to_string();
        let has_to_be_closed = node.kind() != "html_void_tag_open";
        let open_location = node_location(node);
        let mut attributes = Vec::new();
        let mut body = None;
        loop {
            if !self.cursor.goto_next_sibling() {
                return Err(anyhow::anyhow!("html tag is unclosed"));
            }
            let node = self.cursor.node();
            match node.kind() {
                // TODO: html attributes can contain spml tags
                "dynamic_attribute" => attributes.push(self.parse_plain_attribute()?.1),
                "self_closing_tag_end" => break,
                ">" => {
                    if has_to_be_closed {
                        body = Some(self.parse_tag_body()?);
                    }
                    break;
                }
                _ => (),
            };
        }
        let close_location = node_location(self.cursor.node());
        self.cursor.goto_parent();
        return Ok(HtmlNode {
            open_location,
            name,
            attributes,
            body,
            close_location,
        });
    }

    fn parse_text(&mut self) -> Result<TextNode> {
        let mut node = self.cursor.node();
        let start = node.start_position();
        let mut content = node.utf8_text(self.text_bytes)?.to_string();
        loop {
            if !self.cursor.goto_next_sibling() {
                break;
            }
            node = self.cursor.node();
            match node.kind() {
                "text" | "xml_entity" => content.push_str(node.utf8_text(self.text_bytes)?),
                _ => break,
            };
        }
        let end = node.end_position();
        return Ok(TextNode {
            content,
            range: range_from_points(start, end),
        });
    }

    fn parse_error(&mut self) -> Result<ErrorNode> {
        let node = self.cursor.node();
        let start = node.start_position();
        let end = node.end_position();
        let content = node.utf8_text(self.text_bytes)?.to_string();
        return Ok(ErrorNode {
            content,
            range: range_from_points(start, end),
        });
    }

    fn parse_tag_body(&mut self) -> Result<TagBody> {
        let open_location = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("tag is unclosed"));
        }
        let nodes = self.parse_tags()?;
        return Ok(TagBody {
            open_location,
            nodes,
        });
    }

    fn parse_plain_attribute(&mut self) -> Result<(String, ParsedAttribute<PlainAttribute>)> {
        return AttributeParser::plain(self);
    }

    fn parse_spel_attribute(
        &mut self,
        r#type: &TagAttributeType,
    ) -> Result<(String, ParsedAttribute<SpelAttribute>)> {
        return AttributeParser::spel(self, r#type);
    }
}

fn node_location(node: tree_sitter::Node) -> Location {
    let start = node.start_position();
    let end = node.end_position();
    if start.row != end.row {
        // TODO: we need a representation for that!
        log::warn!(
            "tried to create location from multiline node {:?} at {}",
            node,
            std::backtrace::Backtrace::force_capture()
        );
        // This is highly inaccurate!
        return Location::new(start.column, start.row, node.end_byte() - node.start_byte());
    }
    return Location::new(start.column, start.row, end.column - start.column);
}

fn range_from_points(start: tree_sitter::Point, end: tree_sitter::Point) -> Range {
    return Range {
        start: Position {
            line: start.row as u32,
            character: start.column as u32,
        },
        end: Position {
            line: end.row as u32,
            character: end.column as u32,
        },
    };
}

impl Tree {
    pub(crate) fn new(ts: tree_sitter::Tree, text: &String) -> Result<Self> {
        let parser = &mut TreeParser::new(ts.walk(), &text);
        let header = parser.parse_header()?;
        let nodes = parser.parse_tags()?;
        return Ok(Tree { header, nodes });
    }

    pub(crate) fn node_at(&self, position: Position) -> Option<&Node> {
        let mut nodes = &self.nodes;
        let mut current = None;
        loop {
            if let Some(node) = find_tag_at(nodes, position) {
                current = Some(node);
                match node {
                    Node::Tag(tag) => {
                        let tag = match tag {
                            ParsedTag::Valid(tag) => tag,
                            ParsedTag::Erroneous(tag, _) => tag,
                            ParsedTag::Unparsable(_, _) => return current,
                        };
                        if let Some(body) = tag.body() {
                            nodes = &body.nodes;
                            continue;
                        }
                    }
                    Node::Html(tag) => {
                        // TODO: html attributes can contain spml tags
                        if let Some(body) = tag.body() {
                            nodes = &body.nodes;
                            continue;
                        }
                    }
                    _ => (),
                };
            }
            return current;
        }
    }
}

fn find_tag_at(nodes: &Vec<Node>, position: Position) -> Option<&Node> {
    for node in nodes {
        let range = node.range();
        if position > range.end {
            continue;
        }
        if position < range.start {
            break;
        }
        return Some(node);
    }
    return None;
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::{
            Header, Location, Node, PageHeader, ParsedAttribute, ParsedNode, ParsedTag,
            PlainAttribute, PlainAttributeValue, SpBarcode, SpelAttribute, SpelAttributeValue,
            SpmlTag, TagLibImport, TagLibOrigin,
        },
        spel::{
            self,
            ast::{Identifier, SpelAst, SpelResult, StringLiteral, Word, WordFragment},
        },
    };

    use super::{ErrorNode, IncompleteTagLibImport, ParsedLocation, RangedNode, TreeParser};
    use anyhow::{Error, Result};
    use lsp_types::{Position, Range};

    #[test]
    fn test_parse_header() -> Result<()> {
        let document = String::from(concat!(
            "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
            "%><%@ taglib uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n",
            "%><%@ taglib tagdir=\"/WEB-INF/tags/spt\" prefix=\"spt\"\n",
            "%>\n"
        ));
        let expected = Header {
            java_headers: vec![ParsedNode::Valid(PageHeader {
                open_bracket: Location::new(0, 0, 3),
                page: Location::new(4, 0, 4),
                language: Some(ParsedAttribute::Valid(PlainAttribute {
                    key_location: Location::new(9, 0, 8),
                    value: Some(PlainAttributeValue {
                        equals_location: Location::new(17, 0, 1),
                        opening_quote_location: Location::new(18, 0, 1),
                        value: "java".to_string(),
                        closing_quote_location: Location::new(23, 0, 1),
                    }),
                })),
                page_encoding: Some(ParsedAttribute::Valid(PlainAttribute {
                    key_location: Location::new(25, 0, 12),
                    value: Some(PlainAttributeValue {
                        equals_location: Location::new(37, 0, 1),
                        opening_quote_location: Location::new(38, 0, 1),
                        value: "UTF-8".to_string(),
                        closing_quote_location: Location::new(44, 0, 1),
                    }),
                })),
                content_type: Some(ParsedAttribute::Valid(PlainAttribute {
                    key_location: Location::new(46, 0, 11),
                    value: Some(PlainAttributeValue {
                        equals_location: Location::new(57, 0, 1),
                        opening_quote_location: Location::new(58, 0, 1),
                        value: "text/html; charset=UTF-8".to_string(),
                        closing_quote_location: Location::new(83, 0, 1),
                    }),
                })),
                imports: vec![],
                close_bracket: Location::new(0, 1, 2),
            })],
            taglib_imports: vec![
                ParsedNode::Valid(TagLibImport {
                    open_bracket: Location::new(2, 1, 3),
                    taglib: Location::new(6, 1, 6),
                    origin: TagLibOrigin::Uri(ParsedAttribute::Valid(PlainAttribute {
                        key_location: Location::new(13, 1, 3),
                        value: Some(PlainAttributeValue {
                            equals_location: Location::new(16, 1, 1),
                            opening_quote_location: Location::new(17, 1, 1),
                            value: "http://www.sitepark.com/taglibs/core".to_string(),
                            closing_quote_location: Location::new(54, 1, 1),
                        }),
                    })),
                    prefix: ParsedAttribute::Valid(PlainAttribute {
                        key_location: Location::new(56, 1, 6),
                        value: Some(PlainAttributeValue {
                            equals_location: Location::new(62, 1, 1),
                            opening_quote_location: Location::new(63, 1, 1),
                            value: "sp".to_string(),
                            closing_quote_location: Location::new(66, 1, 1),
                        }),
                    }),
                    close_bracket: Location::new(0, 2, 2),
                }),
                ParsedNode::Valid(TagLibImport {
                    open_bracket: Location::new(2, 2, 3),
                    taglib: Location::new(6, 2, 6),
                    origin: TagLibOrigin::TagDir(ParsedAttribute::Valid(PlainAttribute {
                        key_location: Location::new(13, 2, 6),
                        value: Some(PlainAttributeValue {
                            equals_location: Location::new(19, 2, 1),
                            opening_quote_location: Location::new(20, 2, 1),
                            value: "/WEB-INF/tags/spt".to_string(),
                            closing_quote_location: Location::new(38, 2, 1),
                        }),
                    })),
                    prefix: ParsedAttribute::Valid(PlainAttribute {
                        key_location: Location::new(40, 2, 6),
                        value: Some(PlainAttributeValue {
                            equals_location: Location::new(46, 2, 1),
                            opening_quote_location: Location::new(47, 2, 1),
                            value: "spt".to_string(),
                            closing_quote_location: Location::new(51, 2, 1),
                        }),
                    }),
                    close_bracket: Location::new(0, 3, 2),
                }),
            ],
        };
        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser
            .set_language(&tree_sitter_spml::language())
            .map_err(Error::from)?;
        let ts_tree = ts_parser
            .parse(&document, None)
            .ok_or_else(|| anyhow::anyhow!("treesitter parsing failed"))?;
        let parser = &mut TreeParser::new(ts_tree.walk(), &document);
        let header = parser.parse_header()?;
        assert_eq!(header, expected);
        return Ok(());
    }

    #[test]
    fn test_parse_tags() -> Result<()> {
        let document = String::from(concat!(
            "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
            "%>\n",
            "<sp:barcode name=\"_testName\" text=\"some text\" scope=\"page\"/>\n",
        ));
        let expected = vec![Node::Tag(ParsedTag::Valid(SpmlTag::SpBarcode(SpBarcode {
            open_location: Location::new(0, 2, 11),
            height_attribute: None,
            locale_attribute: None,
            name_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                key_location: Location::new(12, 2, 4),
                value: SpelAttributeValue {
                    equals_location: Location::new(16, 2, 1),
                    opening_quote_location: Location::new(17, 2, 1),
                    spel: SpelAst::Identifier(SpelResult::Valid(Identifier::Name(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "_testName".to_string(),
                            location: spel::ast::Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 9,
                            },
                        })],
                    }))),
                    closing_quote_location: Location::new(27, 2, 1),
                },
            })),
            scope_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                key_location: Location::new(46, 2, 5),
                value: SpelAttributeValue {
                    equals_location: Location::new(51, 2, 1),
                    opening_quote_location: Location::new(52, 2, 1),
                    spel: SpelAst::String(SpelResult::Valid(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "page".to_string(),
                            location: spel::ast::Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 4,
                            },
                        })],
                    })),
                    closing_quote_location: Location::new(57, 2, 1),
                },
            })),
            text_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                key_location: Location::new(29, 2, 4),
                value: SpelAttributeValue {
                    equals_location: Location::new(33, 2, 1),
                    opening_quote_location: Location::new(34, 2, 1),
                    spel: SpelAst::String(SpelResult::Valid(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "some text".to_string(),
                            location: spel::ast::Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 9,
                            },
                        })],
                    })),
                    closing_quote_location: Location::new(44, 2, 1),
                },
            })),
            type_attribute: None,
            body: None,
            close_location: Location::new(58, 2, 2),
        })))];
        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser
            .set_language(&tree_sitter_spml::language())
            .map_err(Error::from)?;
        let ts_tree = ts_parser
            .parse(&document, None)
            .ok_or_else(|| anyhow::anyhow!("treesitter parsing failed"))?;
        let parser = &mut TreeParser::new(ts_tree.walk(), &document);
        let _header = parser.parse_header()?;
        let tags = parser.parse_tags()?;
        assert_eq!(tags, expected);
        return Ok(());
    }

    #[test]
    pub fn test_incomplete_taglib_header_range() -> Result<()> {
        let header = IncompleteTagLibImport {
            open_bracket: ParsedLocation::Valid(Location {
                char: 2,
                line: 2,
                length: 3,
            }),
            taglib: ParsedLocation::Missing,
            origin: None,
            prefix: None,
            close_bracket: ParsedLocation::Missing,
            errors: vec![ErrorNode {
                content: "tagli uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n%><%@"
                    .to_string(),
                range: Range {
                    start: Position {
                        line: 2,
                        character: 6,
                    },
                    end: Position {
                        line: 3,
                        character: 5,
                    },
                },
            }],
        };
        let range = header.range();
        let expected = Range {
            start: Position {
                line: 2,
                character: 2,
            },
            end: Position {
                line: 2,
                character: 5,
            },
        };
        assert_eq!(range, Some(expected));
        return Ok(());
    }
}
