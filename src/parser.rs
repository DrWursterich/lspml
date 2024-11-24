#![allow(non_snake_case)]

use std::{cmp::Ordering, str::Utf8Error};

use anyhow::Result;
use lsp_types::{Position, Range};

pub use derive::{DocumentNode, ParsableTag};
use phf::phf_map;

use crate::{
    grammar::{TagAttributeType, TagDefinition},
    spel::{
        self,
        ast::{SpelAst, SpelResult},
    },
};

static TAGS: phf::Map<
    &'static str,
    fn(&mut TreeParser) -> Result<ParsedTag<SpmlTag>, anyhow::Error>,
> = phf_map! {
    "attribute_tag" => |parser| Ok(SpAttribute::parse(parser)?.map(SpmlTag::SpAttribute)),
    "argument_tag" => |parser| Ok(SpArgument::parse(parser)?.map(SpmlTag::SpArgument)),
    "barcode_tag" => |parser| Ok(SpBarcode::parse(parser)?.map(SpmlTag::SpBarcode)),
    "break_tag" => |parser| Ok(SpBreak::parse(parser)?.map(SpmlTag::SpBreak)),
    "calendarsheet_tag" => |parser| Ok( SpCalendarsheet::parse(parser)?.map(SpmlTag::SpCalendarsheet)),
    "checkbox_tag" => |parser| Ok(SpCheckbox::parse(parser)?.map(SpmlTag::SpCheckbox)),
    "code_tag" => |parser| Ok(SpCode::parse(parser)?.map(SpmlTag::SpCode)),
    "collection_tag" => |parser| Ok(SpCollection::parse(parser)?.map(SpmlTag::SpCollection)),
    "condition_tag" => |parser| Ok(SpCondition::parse(parser)?.map(SpmlTag::SpCondition)),
    "diff_tag" => |parser| Ok(SpDiff::parse(parser)?.map(SpmlTag::SpDiff)),
    "else_tag" => |parser| Ok(SpElse::parse(parser)?.map(SpmlTag::SpElse)),
    "elseIf_tag" => |parser| Ok(SpElseIf::parse(parser)?.map(SpmlTag::SpElseIf)),
    "error_tag" => |parser| Ok(SpError::parse(parser)?.map(SpmlTag::SpError)),
    "expire_tag" => |parser| Ok(SpExpire::parse(parser)?.map(SpmlTag::SpExpire)),
    "filter_tag" => |parser| Ok(SpFilter::parse(parser)?.map(SpmlTag::SpFilter)),
    "for_tag" => |parser| Ok(SpFor::parse(parser)?.map(SpmlTag::SpFor)),
    "form_tag" => |parser| Ok(SpForm::parse(parser)?.map(SpmlTag::SpForm)),
    "hidden_tag" => |parser| Ok(SpHidden::parse(parser)?.map(SpmlTag::SpHidden)),
    "if_tag" => |parser| Ok(SpIf::parse(parser)?.map(SpmlTag::SpIf)),
    "include_tag" => |parser| Ok(SpInclude::parse(parser)?.map(SpmlTag::SpInclude)),
    "io_tag" => |parser| Ok(SpIo::parse(parser)?.map(SpmlTag::SpIo)),
    "iterator_tag" => |parser| Ok(SpIterator::parse(parser)?.map(SpmlTag::SpIterator)),
    "json_tag" => |parser| Ok(SpJson::parse(parser)?.map(SpmlTag::SpJson)),
    "linkedinformation_tag" => |parser| Ok(SpLinkedinformation::parse(parser)?.map(SpmlTag::SpLinkedinformation)),
    "linktree_tag" => |parser| Ok(SpLinktree::parse(parser)?.map(SpmlTag::SpLinktree)),
    "livetree_tag" => |parser| Ok(SpLivetree::parse(parser)?.map(SpmlTag::SpLivetree)),
    "log_tag" => |parser| Ok(SpLog::parse(parser)?.map(SpmlTag::SpLog)),
    "login_tag" => |parser| Ok(SpLogin::parse(parser)?.map(SpmlTag::SpLogin)),
    "loop_tag" => |parser| Ok(SpLoop::parse(parser)?.map(SpmlTag::SpLoop)),
    "map_tag" => |parser| Ok(SpMap::parse(parser)?.map(SpmlTag::SpMap)),
    "option_tag" => |parser| Ok(SpOption::parse(parser)?.map(SpmlTag::SpOption)),
    "password_tag" => |parser| Ok(SpPassword::parse(parser)?.map(SpmlTag::SpPassword)),
    "print_tag" => |parser| Ok(SpPrint::parse(parser)?.map(SpmlTag::SpPrint)),
    "querytree_tag" => |parser| Ok(SpQuerytree::parse(parser)?.map(SpmlTag::SpQuerytree)),
    "radio_tag" => |parser| Ok(SpRadio::parse(parser)?.map(SpmlTag::SpRadio)),
    "range_tag" => |parser| Ok(SpRange::parse(parser)?.map(SpmlTag::SpRange)),
    "return_tag" => |parser| Ok(SpReturn::parse(parser)?.map(SpmlTag::SpReturn)),
    "sass_tag" => |parser| Ok(SpSass::parse(parser)?.map(SpmlTag::SpSass)),
    "scaleimage_tag" => |parser| Ok(SpScaleimage::parse(parser)?.map(SpmlTag::SpScaleimage)),
    "scope_tag" => |parser| Ok(SpScope::parse(parser)?.map(SpmlTag::SpScope)),
    "search_tag" => |parser| Ok(SpSearch::parse(parser)?.map(SpmlTag::SpSearch)),
    "select_tag" => |parser| Ok(SpSelect::parse(parser)?.map(SpmlTag::SpSelect)),
    "set_tag" => |parser| Ok(SpSet::parse(parser)?.map(SpmlTag::SpSet)),
    "sort_tag" => |parser| Ok(SpSort::parse(parser)?.map(SpmlTag::SpSort)),
    "subinformation_tag" => |parser| Ok(SpSubinformation::parse(parser)?.map(SpmlTag::SpSubinformation)),
    "tagbody_tag" => |parser| Ok(SpTagbody::parse(parser)?.map(SpmlTag::SpTagbody)),
    "text_tag" => |parser| Ok(SpText::parse(parser)?.map(SpmlTag::SpText)),
    "textarea_tag" => |parser| Ok(SpTextarea::parse(parser)?.map(SpmlTag::SpTextarea)),
    "textimage_tag" => |parser| Ok(SpTextimage::parse(parser)?.map(SpmlTag::SpTextimage)),
    "throw_tag" => |parser| Ok(SpThrow::parse(parser)?.map(SpmlTag::SpThrow)),
    "toggle_tag" => |parser| Ok(SpToggle::parse(parser)?.map(SpmlTag::SpToggle)),
    "upload_tag" => |parser| Ok(SpUpload::parse(parser)?.map(SpmlTag::SpUpload)),
    "url_tag" => |parser| Ok(SpUrl::parse(parser)?.map(SpmlTag::SpUrl)),
    "warning_tag" => |parser| Ok(SpWarning::parse(parser)?.map(SpmlTag::SpWarning)),
    "worklist_tag" => |parser| Ok(SpWorklist::parse(parser)?.map(SpmlTag::SpWorklist)),
    "zip_tag" => |parser| Ok(SpZip::parse(parser)?.map(SpmlTag::SpZip)),
    "spt_counter_tag" => |parser| Ok(SptCounter::parse(parser)?.map(SpmlTag::SptCounter)),
    "spt_date_tag" => |parser| Ok(SptDate::parse(parser)?.map(SpmlTag::SptDate)),
    "spt_diff_tag" => |parser| Ok(SptDiff::parse(parser)?.map(SpmlTag::SptDiff)),
    "spt_email2img_tag" => |parser| Ok(SptEmail2Img::parse(parser)?.map(SpmlTag::SptEmail2Img)),
    "spt_encryptemail_tag" => |parser| Ok(SptEncryptemail::parse(parser)?.map(SpmlTag::SptEncryptemail)),
    "spt_escapeemail_tag" => |parser| Ok(SptEscapeemail::parse(parser)?.map(SpmlTag::SptEscapeemail)),
    "spt_formsolutions_tag" => |parser| Ok(SptFormsolutions::parse(parser)?.map(SpmlTag::SptFormsolutions)),
    "spt_id2url_tag" => |parser| Ok(SptId2Url::parse(parser)?.map(SpmlTag::SptId2Url)),
    "spt_ilink_tag" => |parser| Ok(SptIlink::parse(parser)?.map(SpmlTag::SptIlink)),
    "spt_imageeditor_tag" => |parser| Ok(SptImageeditor::parse(parser)?.map(SpmlTag::SptImageeditor)),
    "spt_imp_tag" => |parser| Ok(SptImp::parse(parser)?.map(SpmlTag::SptImp)),
    "spt_iterator_tag" => |parser| Ok(SptIterator::parse(parser)?.map(SpmlTag::SptIterator)),
    "spt_link_tag" => |parser| Ok(SptLink::parse(parser)?.map(SpmlTag::SptLink)),
    "spt_number_tag" => |parser| Ok(SptNumber::parse(parser)?.map(SpmlTag::SptNumber)),
    "spt_personalization_tag" => |parser| Ok(SptPersonalization::parse(parser)?.map(SpmlTag::SptPersonalization)),
    "spt_prehtml_tag" => |parser| Ok(SptPrehtml::parse(parser)?.map(SpmlTag::SptPrehtml)),
    "spt_smarteditor_tag" => |parser| Ok(SptSmarteditor::parse(parser)?.map(SpmlTag::SptSmarteditor)),
    "spt_spml_tag" => |parser| Ok(SptSpml::parse(parser)?.map(SpmlTag::SptSpml)),
    "spt_text_tag" => |parser| Ok(SptText::parse(parser)?.map(SpmlTag::SptText)),
    "spt_textarea_tag" => |parser| Ok(SptTextarea::parse(parser)?.map(SpmlTag::SptTextarea)),
    "spt_timestamp_tag" => |parser| Ok(SptTimestamp::parse(parser)?.map(SpmlTag::SptTimestamp)),
    "spt_tinymce_tag" => |parser| Ok(SptTinymce::parse(parser)?.map(SpmlTag::SptTinymce)),
    "spt_updown_tag" => |parser| Ok(SptUpdown::parse(parser)?.map(SpmlTag::SptUpdown)),
    "spt_upload_tag" => |parser| Ok(SptUpload::parse(parser)?.map(SpmlTag::SptUpload)),
    "spt_worklist_tag" => |parser| Ok(SptWorklist::parse(parser)?.map(SpmlTag::SptWorklist)),
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

    fn open_location(&self) -> &SingleLineLocation;

    fn close_location(&self) -> &SingleLineLocation;

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
pub(crate) enum ParsedHtml {
    Valid(HtmlNode),
    Erroneous(HtmlNode, Vec<TagError>),
    Unparsable(String, Location),
}

impl DocumentNode for ParsedHtml {
    fn range(&self) -> Range {
        return match &self {
            ParsedHtml::Valid(html) => html.range(),
            ParsedHtml::Erroneous(html, _) => html.range(),
            ParsedHtml::Unparsable(_, location) => location.range(),
        };
    }
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
    pub(crate) open_bracket: SingleLineLocation,
    pub(crate) page: SingleLineLocation,
    pub(crate) language: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) page_encoding: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) content_type: Option<ParsedAttribute<PlainAttribute>>,
    pub(crate) imports: Vec<ParsedAttribute<PlainAttribute>>,
    pub(crate) close_bracket: SingleLineLocation,
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
            .map(|location| location.start())
            .or_else(|| self.language.as_ref().map(|a| a.start()))
            .or_else(|| self.page_encoding.as_ref().map(|a| a.start()))
            .or_else(|| self.content_type.as_ref().map(|a| a.start()))
            .or_else(|| self.imports.first().map(|a| a.start()))
            .or_else(|| {
                self.close_bracket
                    .location()
                    .map(|location| location.start())
            })?;
        let end = self
            .close_bracket
            .location()
            .map(|location| location.end())
            .or_else(|| self.imports.last().map(|a| a.end()))
            .or_else(|| self.content_type.as_ref().map(|a| a.end()))
            .or_else(|| self.page_encoding.as_ref().map(|a| a.end()))
            .or_else(|| self.language.as_ref().map(|a| a.end()))
            .or_else(|| self.page.location().map(|location| location.end()))
            .or_else(|| self.open_bracket.location().map(|location| location.end()))?;
        return Some(Range { start, end });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagLibImport {
    pub(crate) open_bracket: SingleLineLocation,
    pub(crate) taglib: SingleLineLocation,
    pub(crate) origin: TagLibOrigin,
    pub(crate) prefix: ParsedAttribute<PlainAttribute>,
    pub(crate) close_bracket: SingleLineLocation,
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
            .map(|location| location.start())
            .or_else(|| self.taglib.location().map(|location| location.start()))
            .or_else(|| self.origin.as_ref().map(|e| e.start()))
            .or_else(|| self.prefix.as_ref().map(|e| e.start()))
            .or_else(|| {
                self.close_bracket
                    .location()
                    .map(|location| location.start())
            })?;
        let end = self
            .close_bracket
            .location()
            .map(|location| location.end())
            .or_else(|| self.prefix.as_ref().map(|e| e.end()))
            .or_else(|| self.origin.as_ref().map(|e| e.end()))
            .or_else(|| self.taglib.location().map(|location| location.end()))
            .or_else(|| self.open_bracket.location().map(|location| location.end()))?;
        return Some(Range { start, end });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagLibOrigin {
    Uri(ParsedAttribute<PlainAttribute>),
    TagDir(ParsedAttribute<PlainAttribute>),
}

impl TagLibOrigin {
    fn start(&self) -> Position {
        return match &self {
            TagLibOrigin::Uri(uri) => uri.start(),
            TagLibOrigin::TagDir(tagdir) => tagdir.start(),
        };
    }

    fn end(&self) -> Position {
        return match &self {
            TagLibOrigin::Uri(uri) => uri.end(),
            TagLibOrigin::TagDir(tagdir) => tagdir.end(),
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagBody {
    pub(crate) open_location: SingleLineLocation,
    pub(crate) nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) enum Node {
    Tag(ParsedTag<SpmlTag>),
    Html(ParsedHtml),
    Text(TextNode),
    Error(ErrorNode),
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) struct HtmlNode {
    pub(crate) open_location: SingleLineLocation,
    pub(crate) name: String,
    pub(crate) attributes: Vec<ParsedAttribute<HtmlAttribute>>,
    pub(crate) body: Option<TagBody>,
    pub(crate) close_location: SingleLineLocation,
}

impl HtmlNode {
    pub(crate) fn open_location(&self) -> &SingleLineLocation {
        return &self.open_location;
    }

    pub(crate) fn close_location(&self) -> &SingleLineLocation {
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
    fn start(&self) -> Position {
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

    fn end(&self) -> Position {
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
            pub(crate) open_location: SingleLineLocation,
            $(pub(crate) $param: Option<ParsedAttribute<SpelAttribute>>,)*
            pub(crate) body: Option<TagBody>,
            pub(crate) close_location: SingleLineLocation,
        }

        impl Tag for $name {
            fn start(&self) -> Position {
                return self.open_location.start();
            }

            fn end(&self) -> Position {
                return self.close_location.end();
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
    fn start(&self) -> Position;

    fn end(&self) -> Position;
}

pub(crate) trait Attribute {
    fn start(&self) -> Position;

    fn end(&self) -> Position;
}

pub(crate) trait AttributeValue {
    fn opening_quote_location(&self) -> &SingleLineLocation;

    fn closing_quote_location(&self) -> &SingleLineLocation;

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
pub(crate) struct AttributeKey {
    pub(crate) value: String,
    pub(crate) location: SingleLineLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlainAttribute {
    pub(crate) key: AttributeKey,
    pub(crate) value: PlainAttributeValue,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlainAttributeValue {
    pub(crate) equals_location: SingleLineLocation,
    pub(crate) opening_quote_location: SingleLineLocation,
    pub(crate) content: String,
    pub(crate) closing_quote_location: SingleLineLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HtmlAttribute {
    pub(crate) key: AttributeKey,
    pub(crate) value: Option<HtmlAttributeValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HtmlAttributeValue {
    pub(crate) equals_location: SingleLineLocation,
    pub(crate) opening_quote_location: SingleLineLocation,
    pub(crate) content: HtmlAttributeValueContent,
    pub(crate) closing_quote_location: SingleLineLocation,
}

impl Attribute for HtmlAttribute {
    fn start(&self) -> Position {
        return self.key.location.start();
    }

    fn end(&self) -> Position {
        return match &self.value {
            Some(value) => value.closing_quote_location.end(),
            None => self.key.location.end(),
        };
    }
}

impl AttributeValue for HtmlAttributeValue {
    fn opening_quote_location(&self) -> &SingleLineLocation {
        return &self.opening_quote_location;
    }

    fn closing_quote_location(&self) -> &SingleLineLocation {
        return &self.closing_quote_location;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HtmlAttributeValueContent {
    Empty,
    Plain(String),
    Tag(ParsedTag<SpmlTag>),
    Fragmented(Vec<HtmlAttributeValueFragment>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HtmlAttributeValueFragment {
    Plain(String),
    Tag(ParsedTag<SpmlTag>),
}

impl AttributeValue for PlainAttributeValue {
    fn opening_quote_location(&self) -> &SingleLineLocation {
        &self.opening_quote_location
    }

    fn closing_quote_location(&self) -> &SingleLineLocation {
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
    fn start(&self) -> Position {
        return match &self {
            ParsedAttribute::Valid(attribute) => attribute.start(),
            ParsedAttribute::Erroneous(attribute, _) => attribute.start(),
            ParsedAttribute::Unparsable(_, location) => location.start(),
        };
    }

    fn end(&self) -> Position {
        return match &self {
            ParsedAttribute::Valid(attribute) => attribute.end(),
            ParsedAttribute::Erroneous(attribute, _) => attribute.end(),
            ParsedAttribute::Unparsable(_, location) => location.end(),
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
    fn map<T>(self, function: fn(R) -> T) -> ParsedTag<T>
    where
        T: Tag,
    {
        return match self {
            ParsedTag::Valid(tag) => ParsedTag::Valid(function(tag)),
            ParsedTag::Erroneous(tag, errors) => ParsedTag::Erroneous(function(tag), errors),
            ParsedTag::Unparsable(text, location) => ParsedTag::Unparsable(text, location),
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
        return Range { start, end };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagError {
    Superfluous(String, Location),
    Missing(String, Location),
}

impl Attribute for PlainAttribute {
    fn start(&self) -> Position {
        return self.key.location.start();
    }

    fn end(&self) -> Position {
        return self.value.closing_quote_location.end();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpelAttribute {
    pub(crate) key: AttributeKey,
    pub(crate) value: SpelAttributeValue,
}

impl Attribute for SpelAttribute {
    fn start(&self) -> Position {
        return self.key.location.start();
    }

    fn end(&self) -> Position {
        return self.value.closing_quote_location.end();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpelAttributeValue {
    pub(crate) equals_location: SingleLineLocation,
    pub(crate) opening_quote_location: SingleLineLocation,
    pub(crate) spel: SpelAst,
    pub(crate) closing_quote_location: SingleLineLocation,
}

impl AttributeValue for SpelAttributeValue {
    fn opening_quote_location(&self) -> &SingleLineLocation {
        return &self.opening_quote_location;
    }

    fn closing_quote_location(&self) -> &SingleLineLocation {
        return &self.closing_quote_location;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Location {
    SingleLine(SingleLineLocation),
    MultiLine(MultiLineLocation),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SingleLineLocation {
    pub(crate) char: usize,
    pub(crate) line: usize,
    pub(crate) length: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MultiLineLocation {
    pub(crate) start_char: usize,
    pub(crate) start_line: usize,
    pub(crate) end_char: usize,
    pub(crate) end_line: usize,
}

impl Location {
    pub(crate) fn contains(&self, position: &Position) -> bool {
        return match &self {
            Location::SingleLine(location) => location.contains(position),
            Location::MultiLine(location) => location.contains(position),
        };
    }

    pub(crate) fn start(&self) -> Position {
        return match &self {
            Location::SingleLine(location) => location.start(),
            Location::MultiLine(location) => location.start(),
        };
    }

    pub(crate) fn end(&self) -> Position {
        return match &self {
            Location::SingleLine(location) => location.end(),
            Location::MultiLine(location) => location.end(),
        };
    }

    pub(crate) fn range(&self) -> Range {
        return match &self {
            Location::SingleLine(location) => location.range(),
            Location::MultiLine(location) => location.range(),
        };
    }
}

impl SingleLineLocation {
    pub(crate) fn new(char: usize, line: usize, length: usize) -> Self {
        return SingleLineLocation { char, line, length };
    }

    pub(crate) fn contains(&self, position: &Position) -> bool {
        return self.line == position.line as usize
            && self.char <= position.character as usize
            && self.char + self.length > position.character as usize;
    }

    pub(crate) fn start(&self) -> Position {
        return Position {
            line: self.line as u32,
            character: self.char as u32,
        };
    }

    pub(crate) fn end(&self) -> Position {
        return Position {
            line: self.line as u32,
            character: (self.char + self.length) as u32,
        };
    }

    pub(crate) fn range(&self) -> Range {
        return Range {
            start: self.start(),
            end: self.end(),
        };
    }
}

impl MultiLineLocation {
    pub(crate) fn new(
        start_char: usize,
        start_line: usize,
        end_char: usize,
        end_line: usize,
    ) -> Self {
        return MultiLineLocation {
            start_char,
            start_line,
            end_char,
            end_line,
        };
    }

    pub(crate) fn contains(&self, position: &Position) -> bool {
        return match self.start_line.cmp(&(position.line as usize)) {
            Ordering::Greater => false,
            Ordering::Equal => self.start_char <= position.character as usize,
            Ordering::Less => match self.end_line.cmp(&(position.line as usize)) {
                Ordering::Greater => true,
                Ordering::Equal => self.end_char > position.character as usize,
                Ordering::Less => false,
            },
        };
    }

    pub(crate) fn start(&self) -> Position {
        return Position {
            line: self.start_line as u32,
            character: self.start_char as u32,
        };
    }

    pub(crate) fn end(&self) -> Position {
        return Position {
            line: self.end_line as u32,
            character: self.end_char as u32,
        };
    }

    pub(crate) fn range(&self) -> Range {
        return Range {
            start: self.start(),
            end: self.end(),
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
}

enum IntermediateAttributeParsingResult<R> {
    Failed(String, Location),
    Partial(R),
}

impl<'a, 'b> AttributeParser<'a, 'b> {
    fn plain(tree_parser: &'a mut TreeParser<'b>) -> Result<ParsedAttribute<PlainAttribute>> {
        let depth = tree_parser.cursor.depth();
        let parser = AttributeParser::new(tree_parser);
        let result = parser.parse_plain();
        walk_parser_back_to(tree_parser, depth);
        return result;
    }

    fn html(tree_parser: &'a mut TreeParser<'b>) -> Result<ParsedAttribute<HtmlAttribute>> {
        let depth = tree_parser.cursor.depth();
        let parser = AttributeParser::new(tree_parser);
        let result = parser.parse_html();
        walk_parser_back_to(tree_parser, depth);
        return result;
    }

    fn spel(
        tree_parser: &'a mut TreeParser<'b>,
        r#type: &TagAttributeType,
    ) -> Result<ParsedAttribute<SpelAttribute>> {
        let depth = tree_parser.cursor.depth();
        let parser = AttributeParser::new(tree_parser);
        let result = parser.parse_spel(r#type);
        walk_parser_back_to(tree_parser, depth);
        return result;
    }

    fn new(tree_parser: &'a mut TreeParser<'b>) -> Self {
        let parent_node = tree_parser.cursor.node();
        return AttributeParser {
            tree_parser,
            parent_node,
            errors: None,
        };
    }

    fn parse_plain(mut self) -> Result<ParsedAttribute<PlainAttribute>> {
        let key_node = match self.parse_key()? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location));
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let key = match node_location(key_node) {
            Location::SingleLine(location) => AttributeKey {
                value: self.tree_parser.node_text(&key_node)?.to_string(),
                location,
            },
            location => {
                return Ok(ParsedAttribute::Unparsable(
                    "attribute key should be on a single line".to_string(),
                    location,
                ));
            }
        };
        let equals_location = match self.parse_equals(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(Some(e)) => e,
            IntermediateAttributeParsingResult::Partial(None) => {
                return Ok(ParsedAttribute::Unparsable(
                    "missing \"=\"".to_string(),
                    node_location(self.parent_node),
                ))
            }
        };
        if let IntermediateAttributeParsingResult::Failed(message, location) =
            self.parse_string(&key_node)?
        {
            return Ok(ParsedAttribute::Unparsable(message, location));
        }
        let opening_quote_location = match self.parse_opening_quote(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let (content, movement) = match self.parse_string_content(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let closing_quote_location = match self.parse_closing_quote(&key_node, movement)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let value = PlainAttributeValue {
            equals_location,
            opening_quote_location,
            content,
            closing_quote_location,
        };
        let attribute = PlainAttribute { key, value };
        return Ok(match self.errors {
            Some(errors) => ParsedAttribute::Erroneous(attribute, errors),
            None => ParsedAttribute::Valid(attribute),
        });
    }

    fn parse_html(mut self) -> Result<ParsedAttribute<HtmlAttribute>> {
        let key_node = match self.parse_key()? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location));
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let key = match node_location(key_node) {
            Location::SingleLine(location) => AttributeKey {
                value: self.tree_parser.node_text(&key_node)?.to_string(),
                location,
            },
            location => {
                return Ok(ParsedAttribute::Unparsable(
                    "attribute key should be on a single line".to_string(),
                    location,
                ));
            }
        };
        let equals_location = match self.parse_equals(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let value = match equals_location {
            Some(equals_location) => {
                if let IntermediateAttributeParsingResult::Failed(message, location) =
                    self.parse_string(&key_node)?
                {
                    return Ok(ParsedAttribute::Unparsable(message, location));
                }
                let opening_quote_location = match self.parse_opening_quote(&key_node)? {
                    IntermediateAttributeParsingResult::Failed(message, location) => {
                        return Ok(ParsedAttribute::Unparsable(message, location))
                    }
                    IntermediateAttributeParsingResult::Partial(e) => e,
                };
                let content = match self.parse_html_string_content(&key_node)? {
                    IntermediateAttributeParsingResult::Failed(message, location) => {
                        return Ok(ParsedAttribute::Unparsable(message, location))
                    }
                    IntermediateAttributeParsingResult::Partial(e) => e,
                };
                let closing_quote_location =
                    match self.parse_closing_quote(&key_node, NodeMovement::Current)? {
                        IntermediateAttributeParsingResult::Failed(message, location) => {
                            return Ok(ParsedAttribute::Unparsable(message, location))
                        }
                        IntermediateAttributeParsingResult::Partial(e) => e,
                    };
                Some(HtmlAttributeValue {
                    equals_location,
                    opening_quote_location,
                    content,
                    closing_quote_location,
                })
            }
            None => None,
        };
        let attribute = HtmlAttribute { key, value };
        return Ok(match self.errors {
            Some(errors) => ParsedAttribute::Erroneous(attribute, errors),
            None => ParsedAttribute::Valid(attribute),
        });
    }

    fn parse_spel(mut self, r#type: &TagAttributeType) -> Result<ParsedAttribute<SpelAttribute>> {
        let key_node = match self.parse_key()? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location));
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let key = match node_location(key_node) {
            Location::SingleLine(location) => AttributeKey {
                value: self.tree_parser.node_text(&key_node)?.to_string(),
                location,
            },
            location => {
                return Ok(ParsedAttribute::Unparsable(
                    "attribute key should be on a single line".to_string(),
                    location,
                ));
            }
        };
        let equals_location = match self.parse_equals(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(Some(e)) => e,
            IntermediateAttributeParsingResult::Partial(None) => {
                return Ok(ParsedAttribute::Unparsable(
                    "missing \"=\"".to_string(),
                    node_location(self.parent_node),
                ))
            }
        };
        if let IntermediateAttributeParsingResult::Failed(message, location) =
            self.parse_string(&key_node)?
        {
            return Ok(ParsedAttribute::Unparsable(message, location));
        }
        let opening_quote_location = match self.parse_opening_quote(&key_node)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let (spel, movement) = match self.parse_spel_content(&key_node, r#type)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let closing_quote_location = match self.parse_closing_quote(&key_node, movement)? {
            IntermediateAttributeParsingResult::Failed(message, location) => {
                return Ok(ParsedAttribute::Unparsable(message, location))
            }
            IntermediateAttributeParsingResult::Partial(e) => e,
        };
        let attribute = SpelAttribute {
            key,
            value: SpelAttributeValue {
                equals_location,
                opening_quote_location,
                spel,
                closing_quote_location,
            },
        };
        return Ok(match self.errors {
            Some(errors) => ParsedAttribute::Erroneous(attribute, errors),
            None => ParsedAttribute::Valid(attribute),
        });
    }

    fn parse_key(&mut self) -> Result<IntermediateAttributeParsingResult<tree_sitter::Node<'a>>> {
        let mut movement = &NodeMovement::FirstChild;
        loop {
            match self.tree_parser.goto(movement) {
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
    ) -> Result<IntermediateAttributeParsingResult<Option<SingleLineLocation>>> {
        loop {
            return Ok(match self.tree_parser.goto(&NodeMovement::NextSibling) {
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
                NodeMovingResult::Ok(node) => match node_location(node) {
                    Location::SingleLine(location) => {
                        IntermediateAttributeParsingResult::Partial(Some(location))
                    }
                    location => IntermediateAttributeParsingResult::Failed(
                        "\"=\" should be on a single line".to_string(),
                        location,
                    ),
                },
            });
        }
    }

    fn parse_string(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
    ) -> Result<IntermediateAttributeParsingResult<()>> {
        loop {
            return Ok(match self.tree_parser.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "missing attribute value for \"{}\"",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
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
    ) -> Result<IntermediateAttributeParsingResult<SingleLineLocation>> {
        loop {
            return Ok(match self.tree_parser.goto(&NodeMovement::FirstChild) {
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
                NodeMovingResult::Ok(node) => match node_location(node) {
                    Location::SingleLine(location) => {
                        IntermediateAttributeParsingResult::Partial(location)
                    }
                    location => IntermediateAttributeParsingResult::Failed(
                        "\"\"\" should be on a single line".to_string(),
                        location,
                    ),
                },
            });
        }
    }

    fn parse_string_content(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
    ) -> Result<IntermediateAttributeParsingResult<(String, NodeMovement)>> {
        loop {
            return Ok(match self.tree_parser.goto(&NodeMovement::NextSibling) {
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

    fn parse_html_string_content(
        &mut self,
        key_node: &tree_sitter::Node<'a>,
    ) -> Result<IntermediateAttributeParsingResult<HtmlAttributeValueContent>> {
        loop {
            return Ok(match self.tree_parser.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" html attribute value string is unclosed",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Missing(_) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" html attribute value string has no content",
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
                    IntermediateAttributeParsingResult::Partial(HtmlAttributeValueContent::Empty)
                }
                NodeMovingResult::Ok(_) => IntermediateAttributeParsingResult::Partial(
                    self.tree_parser.parse_html_attribute_value_content()?,
                ),
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
    ) -> Result<IntermediateAttributeParsingResult<SingleLineLocation>> {
        loop {
            return Ok(match self.tree_parser.goto(&movement) {
                NodeMovingResult::NonExistent => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" closing attribute value string is unclosed",
                        self.tree_parser.node_text(&key_node)?
                    ),
                    node_location(self.parent_node),
                ),
                NodeMovingResult::Missing(_) => IntermediateAttributeParsingResult::Failed(
                    format!(
                        "\"{}\" closing attribute value string is unclosed",
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
                NodeMovingResult::Ok(node) => match node_location(node) {
                    Location::SingleLine(location) => {
                        IntermediateAttributeParsingResult::Partial(location)
                    }
                    location => IntermediateAttributeParsingResult::Failed(
                        "\"\"\" should be on a single line".to_string(),
                        location,
                    ),
                },
            });
        }
    }

    fn add_error(&mut self, error: AttributeError) {
        match &mut self.errors {
            None => self.errors = Some(vec![error]),
            Some(errors) => errors.push(error),
        }
    }
}

fn walk_parser_back_to(parser: &mut TreeParser, depth: u32) {
    let depth = parser.cursor.depth() - depth;
    for _ in 0..depth {
        parser.cursor.goto_parent();
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

    // TODO: use self.goto()
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
                "import_attribute" => imports.push(self.parse_plain_attribute()?),
                "contentType_attribute" => content_type = Some(self.parse_plain_attribute()?),
                "language_attribute" => language = Some(self.parse_plain_attribute()?),
                "pageEncoding_attribute" => page_encoding = Some(self.parse_plain_attribute()?),
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
                ParsedLocation::Valid(Location::SingleLine(open_bracket)),
                ParsedLocation::Valid(Location::SingleLine(page)),
                ParsedLocation::Valid(Location::SingleLine(close_bracket)),
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

    // TODO: use self.goto()
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
                                            Some(TagLibOrigin::Uri(self.parse_plain_attribute()?))
                                    }
                                    "tagdir_attribute" => {
                                        origin = Some(TagLibOrigin::TagDir(
                                            self.parse_plain_attribute()?,
                                        ))
                                    }
                                    "prefix_attribute" => {
                                        prefix = Some(self.parse_plain_attribute()?)
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
                    ParsedLocation::Valid(Location::SingleLine(open_bracket)),
                    ParsedLocation::Valid(Location::SingleLine(taglib)),
                    Some(origin),
                    Some(prefix),
                    ParsedLocation::Valid(Location::SingleLine(close_bracket)),
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
                "html_tag" | "script_tag" | "style_tag" => {
                    let depth = self.cursor.depth();
                    let html = self.parse_html_tag()?;
                    walk_parser_back_to(self, depth);
                    tags.push(Node::Html(html));
                }
                "html_option_tag" => {
                    let depth = self.cursor.depth();
                    let html = self.parse_html_option_tag()?; // TODO: expectes /> or a body!
                    walk_parser_back_to(self, depth);
                    tags.push(Node::Html(html));
                }
                "html_void_tag" => {
                    let depth = self.cursor.depth();
                    let html = self.parse_html_void_tag()?; // TODO: expectes /> or a body!
                    walk_parser_back_to(self, depth);
                    tags.push(Node::Html(html));
                }
                kind if kind.ends_with("_tag_close") => break,
                kind => {
                    if let Some(mut parse_fn) = TAGS.get(kind).take() {
                        tags.push(Node::Tag(parse_fn.call_mut((self,))?));
                    } else {
                        log::debug!("encountered unexpected tree sitter node {}", kind);
                    }
                }
            };
            if !self.cursor.goto_next_sibling() {
                break;
            }
        }
        return Ok(tags);
    }

    fn parse_html_attribute_value_content(&mut self) -> Result<HtmlAttributeValueContent> {
        let mut fragments: Vec<HtmlAttributeValueFragment> = Vec::new();
        loop {
            let node = self.cursor.node();
            match node.kind() {
                // TODO: these are probably possible though not yet supported by treesitter
                // "comment" => (),
                // "xml_entity" => (),
                "string_content" => {
                    fragments.push(
                        self.parse_text()
                            .map(|text| HtmlAttributeValueFragment::Plain(text.content))?,
                    );
                    continue;
                }
                // "ERROR" => fragments.push(self.parse_error().map(Node::Error)?),
                "attribute_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpAttribute::parse(self)?.map(SpmlTag::SpAttribute),
                )),
                "argument_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpArgument::parse(self)?.map(SpmlTag::SpArgument),
                )),
                "barcode_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpBarcode::parse(self)?.map(SpmlTag::SpBarcode),
                )),
                "break_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpBreak::parse(self)?.map(SpmlTag::SpBreak),
                )),
                "calendarsheet_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpCalendarsheet::parse(self)?.map(SpmlTag::SpCalendarsheet),
                )),
                "checkbox_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpCheckbox::parse(self)?.map(SpmlTag::SpCheckbox),
                )),
                "code_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpCode::parse(self)?.map(SpmlTag::SpCode),
                )),
                "collection_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpCollection::parse(self)?.map(SpmlTag::SpCollection),
                )),
                "condition_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpCondition::parse(self)?.map(SpmlTag::SpCondition),
                )),
                "diff_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpDiff::parse(self)?.map(SpmlTag::SpDiff),
                )),
                "else_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpElse::parse(self)?.map(SpmlTag::SpElse),
                )),
                "elseIf_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpElseIf::parse(self)?.map(SpmlTag::SpElseIf),
                )),
                "error_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpError::parse(self)?.map(SpmlTag::SpError),
                )),
                "expire_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpExpire::parse(self)?.map(SpmlTag::SpExpire),
                )),
                "filter_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpFilter::parse(self)?.map(SpmlTag::SpFilter),
                )),
                "for_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpFor::parse(self)?.map(SpmlTag::SpFor),
                )),
                "form_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpForm::parse(self)?.map(SpmlTag::SpForm),
                )),
                "hidden_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpHidden::parse(self)?.map(SpmlTag::SpHidden),
                )),
                "if_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpIf::parse(self)?.map(SpmlTag::SpIf),
                )),
                "include_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpInclude::parse(self)?.map(SpmlTag::SpInclude),
                )),
                "io_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpIo::parse(self)?.map(SpmlTag::SpIo),
                )),
                "iterator_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpIterator::parse(self)?.map(SpmlTag::SpIterator),
                )),
                "json_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpJson::parse(self)?.map(SpmlTag::SpJson),
                )),
                "linkedinformation_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpLinkedinformation::parse(self)?.map(SpmlTag::SpLinkedinformation),
                )),
                "linktree_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpLinktree::parse(self)?.map(SpmlTag::SpLinktree),
                )),
                "livetree_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpLivetree::parse(self)?.map(SpmlTag::SpLivetree),
                )),
                "log_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpLog::parse(self)?.map(SpmlTag::SpLog),
                )),
                "login_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpLogin::parse(self)?.map(SpmlTag::SpLogin),
                )),
                "loop_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpLoop::parse(self)?.map(SpmlTag::SpLoop),
                )),
                "map_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpMap::parse(self)?.map(SpmlTag::SpMap),
                )),
                "option_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpOption::parse(self)?.map(SpmlTag::SpOption),
                )),
                "password_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpPassword::parse(self)?.map(SpmlTag::SpPassword),
                )),
                "print_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpPrint::parse(self)?.map(SpmlTag::SpPrint),
                )),
                "querytree_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpQuerytree::parse(self)?.map(SpmlTag::SpQuerytree),
                )),
                "radio_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpRadio::parse(self)?.map(SpmlTag::SpRadio),
                )),
                "range_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpRange::parse(self)?.map(SpmlTag::SpRange),
                )),
                "return_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpReturn::parse(self)?.map(SpmlTag::SpReturn),
                )),
                "sass_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpSass::parse(self)?.map(SpmlTag::SpSass),
                )),
                "scaleimage_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpScaleimage::parse(self)?.map(SpmlTag::SpScaleimage),
                )),
                "scope_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpScope::parse(self)?.map(SpmlTag::SpScope),
                )),
                "search_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpSearch::parse(self)?.map(SpmlTag::SpSearch),
                )),
                "select_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpSelect::parse(self)?.map(SpmlTag::SpSelect),
                )),
                "set_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpSet::parse(self)?.map(SpmlTag::SpSet),
                )),
                "sort_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpSort::parse(self)?.map(SpmlTag::SpSort),
                )),
                "subinformation_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpSubinformation::parse(self)?.map(SpmlTag::SpSubinformation),
                )),
                "tagbody_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpTagbody::parse(self)?.map(SpmlTag::SpTagbody),
                )),
                "text_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpText::parse(self)?.map(SpmlTag::SpText),
                )),
                "textarea_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpTextarea::parse(self)?.map(SpmlTag::SpTextarea),
                )),
                "textimage_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpTextimage::parse(self)?.map(SpmlTag::SpTextimage),
                )),
                "throw_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpThrow::parse(self)?.map(SpmlTag::SpThrow),
                )),
                "toggle_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpToggle::parse(self)?.map(SpmlTag::SpToggle),
                )),
                "upload_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpUpload::parse(self)?.map(SpmlTag::SpUpload),
                )),
                "url_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpUrl::parse(self)?.map(SpmlTag::SpUrl),
                )),
                "warning_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpWarning::parse(self)?.map(SpmlTag::SpWarning),
                )),
                "worklist_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpWorklist::parse(self)?.map(SpmlTag::SpWorklist),
                )),
                "zip_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SpZip::parse(self)?.map(SpmlTag::SpZip),
                )),
                "spt_counter_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptCounter::parse(self)?.map(SpmlTag::SptCounter),
                )),
                "spt_date_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptDate::parse(self)?.map(SpmlTag::SptDate),
                )),
                "spt_diff_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptDiff::parse(self)?.map(SpmlTag::SptDiff),
                )),
                "spt_email2img_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptEmail2Img::parse(self)?.map(SpmlTag::SptEmail2Img),
                )),
                "spt_encryptemail_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptEncryptemail::parse(self)?.map(SpmlTag::SptEncryptemail),
                )),
                "spt_escapeemail_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptEscapeemail::parse(self)?.map(SpmlTag::SptEscapeemail),
                )),
                "spt_formsolutions_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptFormsolutions::parse(self)?.map(SpmlTag::SptFormsolutions),
                )),
                "spt_id2url_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptId2Url::parse(self)?.map(SpmlTag::SptId2Url),
                )),
                "spt_ilink_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptIlink::parse(self)?.map(SpmlTag::SptIlink),
                )),
                "spt_imageeditor_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptImageeditor::parse(self)?.map(SpmlTag::SptImageeditor),
                )),
                "spt_imp_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptImp::parse(self)?.map(SpmlTag::SptImp),
                )),
                "spt_iterator_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptIterator::parse(self)?.map(SpmlTag::SptIterator),
                )),
                "spt_link_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptLink::parse(self)?.map(SpmlTag::SptLink),
                )),
                "spt_number_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptNumber::parse(self)?.map(SpmlTag::SptNumber),
                )),
                "spt_personalization_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptPersonalization::parse(self)?.map(SpmlTag::SptPersonalization),
                )),
                "spt_prehtml_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptPrehtml::parse(self)?.map(SpmlTag::SptPrehtml),
                )),
                "spt_smarteditor_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptSmarteditor::parse(self)?.map(SpmlTag::SptSmarteditor),
                )),
                "spt_spml_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptSpml::parse(self)?.map(SpmlTag::SptSpml),
                )),
                "spt_text_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptText::parse(self)?.map(SpmlTag::SptText),
                )),
                "spt_textarea_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptTextarea::parse(self)?.map(SpmlTag::SptTextarea),
                )),
                "spt_timestamp_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptTimestamp::parse(self)?.map(SpmlTag::SptTimestamp),
                )),
                "spt_tinymce_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptTinymce::parse(self)?.map(SpmlTag::SptTinymce),
                )),
                "spt_updown_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptUpdown::parse(self)?.map(SpmlTag::SptUpdown),
                )),
                "spt_upload_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptUpload::parse(self)?.map(SpmlTag::SptUpload),
                )),
                "spt_worklist_tag" => fragments.push(HtmlAttributeValueFragment::Tag(
                    SptWorklist::parse(self)?.map(SpmlTag::SptWorklist),
                )),
                "\"" => break,
                kind => log::debug!("encountered unexpected tree sitter node {}", kind),
            };
            if !self.cursor.goto_next_sibling() {
                break;
            }
        }
        return Ok(match fragments.len() {
            0 => HtmlAttributeValueContent::Empty,
            1 => match fragments[0].to_owned() {
                HtmlAttributeValueFragment::Plain(text) => HtmlAttributeValueContent::Plain(text),
                HtmlAttributeValueFragment::Tag(tag) => HtmlAttributeValueContent::Tag(tag),
            },
            _ => HtmlAttributeValueContent::Fragmented(fragments),
        });
    }

    fn parse_html_tag(&mut self) -> Result<ParsedHtml> {
        let parent_node = self.cursor.node();
        let mut errors = Vec::new();
        let mut movement = &NodeMovement::FirstChild;
        let node;
        loop {
            node = match self.goto(movement) {
                NodeMovingResult::NonExistent | NodeMovingResult::Missing(_) => {
                    // return Err(anyhow::anyhow!("html tag is empty"));
                    return Ok(ParsedHtml::Unparsable(
                        "missing html".to_string(),
                        node_location(parent_node),
                    ));
                }
                NodeMovingResult::Erroneous(node) => {
                    return Ok(ParsedHtml::Unparsable(
                        format!("invalid html \"{}\"", self.node_text(&node)?),
                        node_location(node),
                    ));
                }
                NodeMovingResult::Superfluous(node) => {
                    errors.push(TagError::Superfluous(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    movement = &NodeMovement::NextSibling;
                    continue;
                }
                NodeMovingResult::Ok(node) => node,
            };
            break;
        }
        let name = node.utf8_text(self.text_bytes)?;
        let name = name.strip_prefix("<").unwrap_or(&name).to_string();
        let open_location = match node_location(node) {
            Location::SingleLine(location) => location,
            _ => return Err(anyhow::anyhow!("\"<{}\" should be on a single line", name)),
        };
        let mut attributes = Vec::new();
        let mut body = None;
        let close_location;
        loop {
            let node = match self.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => {
                    return Ok(ParsedHtml::Unparsable(
                        "html tag is unclosed".to_string(),
                        node_location(parent_node),
                    ));
                }
                NodeMovingResult::Missing(node) => {
                    errors.push(TagError::Missing(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    if node.kind() == "dynamic_attribute" {
                        continue;
                    }
                    node
                }
                NodeMovingResult::Erroneous(node) => {
                    return Ok(ParsedHtml::Unparsable(
                        format!("invalid html \"{}\"", self.node_text(&node)?),
                        node_location(node),
                    ));
                }
                NodeMovingResult::Superfluous(node) => {
                    errors.push(TagError::Superfluous(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(node) => {
                    if node.kind() == "dynamic_attribute" {
                        attributes.push(AttributeParser::html(self)?);
                        continue;
                    }
                    node
                }
            };
            close_location = match node.kind() {
                "self_closing_tag_end" => {
                    let mut movement = &NodeMovement::Current;
                    let node;
                    loop {
                        node = match self.goto(movement) {
                            NodeMovingResult::NonExistent => {
                                return Err(anyhow::anyhow!("current node cannot be non-existent"));
                            }
                            NodeMovingResult::Missing(node) => {
                                // tree-sitter puts missing "/>" nodes always at the first possible
                                // location. in order for completion to work we instead want it to
                                // include all following whitespace, so we search for the next node
                                // and place it in front of it. if this is the last node we have to
                                // manually split the documents text to find "trailing" whitespace,
                                // which is not included in any node.
                                // however, the error reported must still be on the first possible
                                // location such that the quick-fix action inserts it there.
                                errors
                                    .push(TagError::Missing("/>".to_string(), node_location(node)));
                                self.move_missing_node_past_whitespaces(node)?
                            }
                            NodeMovingResult::Erroneous(node) => {
                                return Ok(ParsedHtml::Unparsable(
                                    format!("invalid html \"{}\"", self.node_text(&node)?),
                                    node_location(node),
                                ));
                            }
                            NodeMovingResult::Superfluous(node) => {
                                errors.push(TagError::Superfluous(
                                    self.node_text(&node)?.to_string(),
                                    node_location(node),
                                ));
                                movement = &NodeMovement::NextSibling;
                                continue;
                            }
                            NodeMovingResult::Ok(node) => node_location(node),
                        };
                        break;
                    }
                    node
                }
                ">" => {
                    body = Some(match self.parse_tag_body()? {
                        Some(body) => body,
                        None => {
                            return Ok(ParsedHtml::Unparsable(
                                format!("html tag \"{}\" is unclosed", name),
                                node_location(node),
                            ))
                        }
                    });
                    let mut movement = &NodeMovement::Current;
                    let location;
                    loop {
                        location = match self.goto(movement) {
                            NodeMovingResult::NonExistent => {
                                return Err(anyhow::anyhow!("current node cannot be non-existent"));
                            }
                            NodeMovingResult::Missing(node) => {
                                let location = node_location(node);
                                errors.push(TagError::Missing(
                                    format!("</{}>", name),
                                    // tree-sitter puts missing nodes always at the very end!
                                    location.clone(),
                                ));
                                location
                            }
                            NodeMovingResult::Erroneous(node) => {
                                return Ok(ParsedHtml::Unparsable(
                                    format!("invalid html \"{}\"", self.node_text(&node)?),
                                    node_location(node),
                                ));
                            }
                            NodeMovingResult::Superfluous(node) => {
                                errors.push(TagError::Superfluous(
                                    self.node_text(&node)?.to_string(),
                                    node_location(node),
                                ));
                                movement = &NodeMovement::NextSibling;
                                continue;
                            }
                            NodeMovingResult::Ok(node) => node_location(node),
                        };
                        break;
                    }
                    location
                }
                kind => {
                    return Err(anyhow::anyhow!(
                        "encountered unknown node \"{}\" inside of html tag",
                        kind
                    ))
                }
            };
            break;
        }
        let close_location = match close_location {
            Location::SingleLine(location) => location,
            _ => {
                return Err(anyhow::anyhow!(
                    "\"{}\" should be on a single line",
                    node.utf8_text(self.text_bytes)?
                ))
            }
        };
        let html = HtmlNode {
            open_location,
            name,
            attributes,
            body,
            close_location,
        };
        return Ok(match errors.is_empty() {
            true => ParsedHtml::Valid(html),
            false => ParsedHtml::Erroneous(html, errors),
        });
    }

    fn parse_html_option_tag(&mut self) -> Result<ParsedHtml> {
        let parent_node = self.cursor.node();
        let mut errors = Vec::new();
        let mut movement = &NodeMovement::FirstChild;
        let node;
        loop {
            node = match self.goto(movement) {
                NodeMovingResult::NonExistent | NodeMovingResult::Missing(_) => {
                    return Ok(ParsedHtml::Unparsable(
                        "missing html".to_string(),
                        node_location(parent_node),
                    ));
                }
                NodeMovingResult::Erroneous(node) => {
                    return Ok(ParsedHtml::Unparsable(
                        format!("invalid html \"{}\"", self.node_text(&node)?),
                        node_location(node),
                    ));
                }
                NodeMovingResult::Superfluous(node) => {
                    errors.push(TagError::Superfluous(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    movement = &NodeMovement::NextSibling;
                    continue;
                }
                NodeMovingResult::Ok(node) => node,
            };
            break;
        }
        let name = node.utf8_text(self.text_bytes)?;
        let name = name.strip_prefix("<").unwrap_or(&name).to_string();
        let open_location = match node_location(node) {
            Location::SingleLine(location) => location,
            _ => return Err(anyhow::anyhow!("\"<{}\" should be on a single line", name)),
        };
        let mut attributes = Vec::new();
        loop {
            match self.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => {
                    return Ok(ParsedHtml::Unparsable(
                        "html tag is unclosed".to_string(),
                        node_location(parent_node),
                    ));
                }
                NodeMovingResult::Missing(node) => {
                    errors.push(TagError::Missing(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    if node.kind() == "dynamic_attribute" {
                        continue;
                    }
                }
                NodeMovingResult::Erroneous(node) => {
                    return Ok(ParsedHtml::Unparsable(
                        format!("invalid html \"{}\"", self.node_text(&node)?),
                        node_location(node),
                    ));
                }
                NodeMovingResult::Superfluous(node) => {
                    errors.push(TagError::Superfluous(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(node) => {
                    if node.kind() == "dynamic_attribute" {
                        attributes.push(AttributeParser::html(self)?);
                        continue;
                    }
                }
            }
            break;
        }
        let node = self.cursor.node();
        if node.kind() != ">" {
            return Err(anyhow::anyhow!(
                "expected to be at \">\" node, found \"{}\" instead",
                node.kind(),
            ));
        }
        let body = self.parse_tag_body()?;
        let close_location = match body {
            Some(_) => {
                let mut movement = &NodeMovement::Current;
                let close_location;
                loop {
                    close_location = match self.goto(movement) {
                        NodeMovingResult::NonExistent => node_location(node),
                        NodeMovingResult::Missing(node) => {
                            // these should not be able to be missing, they're allowed to
                            let location = node_location(node);
                            errors
                                .push(TagError::Missing(format!("</{}>", name), location.clone()));
                            location
                        }
                        NodeMovingResult::Erroneous(node) => {
                            return Ok(ParsedHtml::Unparsable(
                                format!("invalid html \"{}\"", self.node_text(&node)?),
                                node_location(node),
                            ));
                        }
                        NodeMovingResult::Superfluous(node) => {
                            errors.push(TagError::Superfluous(
                                self.node_text(&node)?.to_string(),
                                node_location(node),
                            ));
                            movement = &NodeMovement::NextSibling;
                            continue;
                        }
                        NodeMovingResult::Ok(node) => node_location(node),
                    };
                    break;
                }
                close_location
            }
            None => node_location(node),
        };
        let close_location = match close_location {
            Location::SingleLine(location) => location,
            _ => {
                return Err(anyhow::anyhow!(
                    "\"{}\" should be on a single line",
                    node.utf8_text(self.text_bytes)?
                ))
            }
        };
        let html = HtmlNode {
            open_location,
            name,
            attributes,
            body,
            close_location,
        };
        return Ok(match errors.is_empty() {
            true => ParsedHtml::Valid(html),
            false => ParsedHtml::Erroneous(html, errors),
        });
    }

    fn parse_html_void_tag(&mut self) -> Result<ParsedHtml> {
        let parent_node = self.cursor.node();
        let mut errors = Vec::new();
        let mut movement = &NodeMovement::FirstChild;
        let node;
        loop {
            node = match self.goto(movement) {
                NodeMovingResult::NonExistent | NodeMovingResult::Missing(_) => {
                    return Ok(ParsedHtml::Unparsable(
                        "missing html".to_string(),
                        node_location(parent_node),
                    ));
                }
                NodeMovingResult::Erroneous(node) => {
                    return Ok(ParsedHtml::Unparsable(
                        format!("invalid html \"{}\"", self.node_text(&node)?),
                        node_location(node),
                    ));
                }
                NodeMovingResult::Superfluous(node) => {
                    errors.push(TagError::Superfluous(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    movement = &NodeMovement::NextSibling;
                    continue;
                }
                NodeMovingResult::Ok(node) => node,
            };
            break;
        }
        let name = node.utf8_text(self.text_bytes)?;
        let name = name.strip_prefix("<").unwrap_or(&name).to_string();
        let open_location = match node_location(node) {
            Location::SingleLine(location) => location,
            _ => return Err(anyhow::anyhow!("\"<{}\" should be on a single line", name)),
        };
        let mut attributes = Vec::new();
        loop {
            match self.goto(&NodeMovement::NextSibling) {
                NodeMovingResult::NonExistent => {
                    return Ok(ParsedHtml::Unparsable(
                        "html tag is unclosed".to_string(),
                        node_location(parent_node),
                    ));
                }
                NodeMovingResult::Missing(node) => {
                    errors.push(TagError::Missing(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    if node.kind() == "dynamic_attribute" {
                        continue;
                    }
                }
                NodeMovingResult::Erroneous(node) => {
                    return Ok(ParsedHtml::Unparsable(
                        format!("invalid html \"{}\"", self.node_text(&node)?),
                        node_location(node),
                    ));
                }
                NodeMovingResult::Superfluous(node) => {
                    errors.push(TagError::Superfluous(
                        self.node_text(&node)?.to_string(),
                        node_location(node),
                    ));
                    continue;
                }
                NodeMovingResult::Ok(node) => {
                    if node.kind() == "dynamic_attribute" {
                        attributes.push(AttributeParser::html(self)?);
                        continue;
                    }
                }
            }
            break;
        }
        let node = self.cursor.node();
        let kind = node.kind();
        if kind != ">" && kind != "self_closing_tag_end" {
            return Err(anyhow::anyhow!(
                "expected to be at \">\" or \"/>\" node, found \"{}\" instead",
                node.kind(),
            ));
        }
        let close_location = match node_location(node) {
            Location::SingleLine(location) => location,
            _ => {
                return Err(anyhow::anyhow!(
                    "\"{}\" should be on a single line",
                    node.utf8_text(self.text_bytes)?
                ))
            }
        };
        let html = HtmlNode {
            open_location,
            name,
            attributes,
            body: None,
            close_location,
        };
        return Ok(match errors.is_empty() {
            true => ParsedHtml::Valid(html),
            false => ParsedHtml::Erroneous(html, errors),
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

    fn parse_tag_body(&mut self) -> Result<Option<TagBody>> {
        let node = self.cursor.node();
        let open_location = match node_location(node) {
            Location::SingleLine(location) => location,
            _ => return Err(anyhow::anyhow!("\"<\" should be on a single line")),
        };
        if !self.cursor.goto_next_sibling() {
            // return Err(anyhow::anyhow!("tag is unclosed"));
            return Ok(None);
        }
        let nodes = self.parse_tags()?;
        return Ok(Some(TagBody {
            open_location,
            nodes,
        }));
    }

    fn parse_plain_attribute(&mut self) -> Result<ParsedAttribute<PlainAttribute>> {
        return AttributeParser::plain(self);
    }

    fn parse_spel_attribute(
        &mut self,
        r#type: &TagAttributeType,
    ) -> Result<ParsedAttribute<SpelAttribute>> {
        return AttributeParser::spel(self, r#type);
    }

    fn move_missing_node_past_whitespaces(&self, node: tree_sitter::Node<'_>) -> Result<Location> {
        let (char, line) = match find_next_node(node) {
            Some(node) => {
                let node_start = node.start_position();
                (node_start.column, node_start.row)
            }
            None => {
                let trailing_text =
                    std::str::from_utf8(self.text_bytes.split_at(node.end_byte()).1)?;
                let mut trailing_lines = trailing_text.lines().peekable();
                let mut lines = 0;
                let mut chars = 0;
                while let Some(line) = trailing_lines.next() {
                    lines += 1;
                    if trailing_lines.peek().is_none() {
                        chars = line.len();
                        break;
                    }
                }
                let node_end = node.end_position();
                (
                    match lines {
                        1 => node_end.column + chars,
                        _ => chars,
                    },
                    node_end.row + lines - 1,
                )
            }
        };
        return Ok(Location::SingleLine(SingleLineLocation {
            char,
            line,
            length: 0,
        }));
    }
}

fn find_next_node(current: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    return current
        .next_sibling()
        .filter(|node| !node.is_missing())
        .or_else(|| current.parent().and_then(find_next_node));
}

fn node_location(node: tree_sitter::Node) -> Location {
    let start = node.start_position();
    let end = node.end_position();
    if start.row != end.row {
        return Location::MultiLine(MultiLineLocation::new(
            start.column,
            start.row,
            end.column,
            end.row,
        ));
    }
    return Location::SingleLine(SingleLineLocation::new(
        start.column,
        start.row,
        end.column - start.column,
    ));
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

    pub(crate) fn node_at<'a>(&'a self, position: Position) -> Option<&'a Node> {
        let mut nodes = &self.nodes;
        let mut current = None;
        loop {
            if let Some(node) = find_node_at(nodes, position) {
                current = Some(node);
                let body = match node {
                    Node::Tag(tag) => match tag {
                        ParsedTag::Valid(tag) => tag.body(),
                        ParsedTag::Erroneous(tag, _) => tag.body(),
                        ParsedTag::Unparsable(_, _) => &None,
                    },
                    Node::Html(html) => match html {
                        ParsedHtml::Valid(tag) => tag.body(),
                        ParsedHtml::Erroneous(tag, _) => tag.body(),
                        ParsedHtml::Unparsable(_, _) => &None,
                    },
                    _ => &None,
                };
                if let Some(body) = body {
                    nodes = &body.nodes;
                    continue;
                }
            }
            return current;
        }
    }

    pub(crate) fn find_tag_in_attributes<'a>(
        &self,
        tag: &'a HtmlNode,
        position: Position,
    ) -> Option<&'a ParsedTag<SpmlTag>> {
        for attribute in &tag.attributes {
            let attribute = match attribute {
                ParsedAttribute::Valid(ref attribute) => attribute,
                ParsedAttribute::Erroneous(ref attribute, _) => attribute,
                ParsedAttribute::Unparsable(_, _) => continue,
            };
            if let Some(value) = &attribute.value {
                if position > value.closing_quote_location.start() {
                    continue;
                }
                if position < value.opening_quote_location.end() {
                    match &value.content {
                        HtmlAttributeValueContent::Tag(tag) => {
                            return Some(tag);
                            // return match tag {
                            //     ParsedTag::Valid(tag) => Some(tag),
                            //     ParsedTag::Erroneous(tag, _) => Some(tag),
                            //     ParsedTag::Unparsable(_, _) => None,
                            // }
                        }
                        HtmlAttributeValueContent::Fragmented(fragments) => {
                            for fragment in fragments {
                                let parsed = match fragment {
                                    HtmlAttributeValueFragment::Tag(parsed) => parsed,
                                    _ => continue,
                                };
                                let tag = match parsed {
                                    ParsedTag::Valid(tag) => Some(tag),
                                    ParsedTag::Erroneous(tag, _) => Some(tag),
                                    ParsedTag::Unparsable(_, _) => None,
                                };
                                if let Some(tag) = tag {
                                    if position > tag.close_location().start() {
                                        continue;
                                    }
                                    if position < tag.open_location().end() {
                                        return Some(parsed);
                                    }
                                }
                                continue;
                            }
                        }
                        _ => break,
                    }
                }
                break;
            }
            continue;
        }
        return None;
    }

    pub(crate) fn parent_of(&self, node: &Node) -> Option<&Node> {
        let position = node.range().start;
        let mut nodes = &self.nodes;
        let mut current = None;
        loop {
            if let Some(node) = find_node_at(nodes, position) {
                if node.range().start != position {
                    current = Some(node);
                    let body = match node {
                        Node::Tag(tag) => match tag {
                            ParsedTag::Valid(tag) => tag.body(),
                            ParsedTag::Erroneous(tag, _) => tag.body(),
                            ParsedTag::Unparsable(_, _) => return current,
                        },
                        Node::Html(tag) => match tag {
                            ParsedHtml::Valid(tag) => tag.body(),
                            ParsedHtml::Erroneous(tag, _) => tag.body(),
                            ParsedHtml::Unparsable(_, _) => return current,
                            // TODO: html attributes can contain spml tags
                        },
                        _ => &None,
                    };
                    if let Some(body) = body {
                        nodes = &body.nodes;
                        continue;
                    }
                }
            }
            return current;
        }
    }
}

fn find_node_at(nodes: &Vec<Node>, position: Position) -> Option<&Node> {
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
            AttributeKey, AttributeParser, Header, HtmlAttribute, HtmlAttributeValue,
            HtmlAttributeValueContent, HtmlNode, Location, Node, NodeMovement, PageHeader,
            ParsedAttribute, ParsedHtml, ParsedNode, ParsedTag, PlainAttribute,
            PlainAttributeValue, SingleLineLocation, SpBarcode, SpPrint, SpelAttribute,
            SpelAttributeValue, SpmlTag, TagLibImport, TagLibOrigin,
        },
        spel::{
            self,
            ast::{Identifier, Object, SpelAst, SpelResult, StringLiteral, Word, WordFragment},
        },
    };

    use super::{
        ErrorNode, HtmlAttributeValueFragment, IncompleteTagLibImport, ParsedLocation, RangedNode,
        TreeParser,
    };
    use anyhow::{Error, Result};
    use lsp_types::{Position, Range};

    #[test]
    fn test_no_stack_overflow_when_deeply_nested() -> Result<()> {
        // apparently 12 levels of nesting is the maximum before a stack overflow
        let document = String::from(concat!(
            "<%@page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/htm/>/>l; charset=UTF-8\"\n",
            "%><%@ taglib uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n",
            "%><%@ taglib tagdir=\"/WEB-INF/tags/spt\" prefix=\"spt\"\n",
            "%>\n",
            "<sp:condition>\n",
            "\t<sp:if name=\"_test\" neq=\"_test1\">\n",
            "\t\t<sp:condition>\n",
            "\t\t\t<sp:if name=\"_test\" neq=\"_test3\">\n",
            "\t\t\t\t<sp:condition>\n",
            "\t\t\t\t\t<sp:if name=\"_test\" neq=\"_test4\">\n",
            "\t\t\t\t\t\t<sp:condition>\n",
            "\t\t\t\t\t\t\t<sp:if name=\"_test\" neq=\"_test5\">\n",
            "\t\t\t\t\t\t\t\t<sp:condition>\n",
            "\t\t\t\t\t\t\t\t\t<sp:if name=\"_test\" neq=\"_test5\">\n",
            "\t\t\t\t\t\t\t\t\t\t<sp:condition>\n",
            "\t\t\t\t\t\t\t\t\t\t\t<sp:if name=\"_test\" neq=\"_test6\">\n",
            "\t\t\t\t\t\t\t\t\t\t\t\t<sp:print value=\"success!\"\n",
            "\t\t\t\t\t\t\t\t\t\t\t</sp:if>\n",
            "\t\t\t\t\t\t\t\t\t\t</sp:condition>\n",
            "\t\t\t\t\t\t\t\t\t</sp:if>\n",
            "\t\t\t\t\t\t\t\t</sp:condition>\n",
            "\t\t\t\t\t\t\t</sp:if>\n",
            "\t\t\t\t\t\t</sp:condition>\n",
            "\t\t\t\t\t</sp:if>\n",
            "\t\t\t\t</sp:condition>\n",
            "\t\t\t</sp:if>\n",
            "\t\t</sp:condition>\n",
            "\t</sp:if>\n",
            "</sp:condition>\n",
        ));
        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser
            .set_language(&tree_sitter_spml::language())
            .map_err(Error::from)?;
        let ts_tree = ts_parser
            .parse(&document, None)
            .ok_or_else(|| anyhow::anyhow!("treesitter parsing failed"))?;
        let parser = &mut TreeParser::new(ts_tree.walk(), &document);
        let _header = parser.parse_header()?;
        let _tags = parser.parse_tags()?;
        return Ok(());
    }

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
                open_bracket: SingleLineLocation::new(0, 0, 3),
                page: SingleLineLocation::new(4, 0, 4),
                language: Some(ParsedAttribute::Valid(PlainAttribute {
                    key: AttributeKey {
                        value: "language".to_string(),
                        location: SingleLineLocation::new(9, 0, 8),
                    },
                    value: PlainAttributeValue {
                        equals_location: SingleLineLocation::new(17, 0, 1),
                        opening_quote_location: SingleLineLocation::new(18, 0, 1),
                        content: "java".to_string(),
                        closing_quote_location: SingleLineLocation::new(23, 0, 1),
                    },
                })),
                page_encoding: Some(ParsedAttribute::Valid(PlainAttribute {
                    key: AttributeKey {
                        value: "pageEncoding".to_string(),
                        location: SingleLineLocation::new(25, 0, 12),
                    },
                    value: PlainAttributeValue {
                        equals_location: SingleLineLocation::new(37, 0, 1),
                        opening_quote_location: SingleLineLocation::new(38, 0, 1),
                        content: "UTF-8".to_string(),
                        closing_quote_location: SingleLineLocation::new(44, 0, 1),
                    },
                })),
                content_type: Some(ParsedAttribute::Valid(PlainAttribute {
                    key: AttributeKey {
                        value: "contentType".to_string(),
                        location: SingleLineLocation::new(46, 0, 11),
                    },
                    value: PlainAttributeValue {
                        equals_location: SingleLineLocation::new(57, 0, 1),
                        opening_quote_location: SingleLineLocation::new(58, 0, 1),
                        content: "text/html; charset=UTF-8".to_string(),
                        closing_quote_location: SingleLineLocation::new(83, 0, 1),
                    },
                })),
                imports: vec![],
                close_bracket: SingleLineLocation::new(0, 1, 2),
            })],
            taglib_imports: vec![
                ParsedNode::Valid(TagLibImport {
                    open_bracket: SingleLineLocation::new(2, 1, 3),
                    taglib: SingleLineLocation::new(6, 1, 6),
                    origin: TagLibOrigin::Uri(ParsedAttribute::Valid(PlainAttribute {
                        key: AttributeKey {
                            value: "uri".to_string(),
                            location: SingleLineLocation::new(13, 1, 3),
                        },
                        value: PlainAttributeValue {
                            equals_location: SingleLineLocation::new(16, 1, 1),
                            opening_quote_location: SingleLineLocation::new(17, 1, 1),
                            content: "http://www.sitepark.com/taglibs/core".to_string(),
                            closing_quote_location: SingleLineLocation::new(54, 1, 1),
                        },
                    })),
                    prefix: ParsedAttribute::Valid(PlainAttribute {
                        key: AttributeKey {
                            value: "prefix".to_string(),
                            location: SingleLineLocation::new(56, 1, 6),
                        },
                        value: PlainAttributeValue {
                            equals_location: SingleLineLocation::new(62, 1, 1),
                            opening_quote_location: SingleLineLocation::new(63, 1, 1),
                            content: "sp".to_string(),
                            closing_quote_location: SingleLineLocation::new(66, 1, 1),
                        },
                    }),
                    close_bracket: SingleLineLocation::new(0, 2, 2),
                }),
                ParsedNode::Valid(TagLibImport {
                    open_bracket: SingleLineLocation::new(2, 2, 3),
                    taglib: SingleLineLocation::new(6, 2, 6),
                    origin: TagLibOrigin::TagDir(ParsedAttribute::Valid(PlainAttribute {
                        key: AttributeKey {
                            value: "tagdir".to_string(),
                            location: SingleLineLocation::new(13, 2, 6),
                        },
                        value: PlainAttributeValue {
                            equals_location: SingleLineLocation::new(19, 2, 1),
                            opening_quote_location: SingleLineLocation::new(20, 2, 1),
                            content: "/WEB-INF/tags/spt".to_string(),
                            closing_quote_location: SingleLineLocation::new(38, 2, 1),
                        },
                    })),
                    prefix: ParsedAttribute::Valid(PlainAttribute {
                        key: AttributeKey {
                            value: "prefix".to_string(),
                            location: SingleLineLocation::new(40, 2, 6),
                        },
                        value: PlainAttributeValue {
                            equals_location: SingleLineLocation::new(46, 2, 1),
                            opening_quote_location: SingleLineLocation::new(47, 2, 1),
                            content: "spt".to_string(),
                            closing_quote_location: SingleLineLocation::new(51, 2, 1),
                        },
                    }),
                    close_bracket: SingleLineLocation::new(0, 3, 2),
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
            open_location: SingleLineLocation::new(0, 2, 11),
            height_attribute: None,
            locale_attribute: None,
            name_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                key: AttributeKey {
                    value: "name".to_string(),
                    location: SingleLineLocation::new(12, 2, 4),
                },
                value: SpelAttributeValue {
                    equals_location: SingleLineLocation::new(16, 2, 1),
                    opening_quote_location: SingleLineLocation::new(17, 2, 1),
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
                    closing_quote_location: SingleLineLocation::new(27, 2, 1),
                },
            })),
            scope_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                key: AttributeKey {
                    value: "scope".to_string(),
                    location: SingleLineLocation::new(46, 2, 5),
                },
                value: SpelAttributeValue {
                    equals_location: SingleLineLocation::new(51, 2, 1),
                    opening_quote_location: SingleLineLocation::new(52, 2, 1),
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
                    closing_quote_location: SingleLineLocation::new(57, 2, 1),
                },
            })),
            text_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                key: AttributeKey {
                    value: "text".to_string(),
                    location: SingleLineLocation::new(29, 2, 4),
                },
                value: SpelAttributeValue {
                    equals_location: SingleLineLocation::new(33, 2, 1),
                    opening_quote_location: SingleLineLocation::new(34, 2, 1),
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
                    closing_quote_location: SingleLineLocation::new(44, 2, 1),
                },
            })),
            type_attribute: None,
            body: None,
            close_location: SingleLineLocation::new(58, 2, 2),
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
            open_bracket: ParsedLocation::Valid(Location::SingleLine(SingleLineLocation::new(
                2, 2, 3,
            ))),
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

    #[test]
    pub fn test_tag_in_html_attribute() -> Result<()> {
        let document = String::from(concat!(
            "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
            "%>\n",
            "<div class=\"<sp:print name=\"_class\"/> centered\">",
        ));
        let expected = ParsedAttribute::Valid(HtmlAttribute {
            key: AttributeKey {
                value: "class".to_string(),
                location: SingleLineLocation::new(5, 2, 5),
            },
            value: Some(HtmlAttributeValue {
                equals_location: SingleLineLocation::new(10, 2, 1),
                opening_quote_location: SingleLineLocation::new(11, 2, 1),
                content: HtmlAttributeValueContent::Fragmented(vec![
                    HtmlAttributeValueFragment::Tag(ParsedTag::Valid(SpmlTag::SpPrint(SpPrint {
                        open_location: SingleLineLocation::new(12, 2, 9),
                        arg_attribute: None,
                        condition_attribute: None,
                        convert_attribute: None,
                        cryptkey_attribute: None,
                        dateformat_attribute: None,
                        decimalformat_attribute: None,
                        decoding_attribute: None,
                        decrypt_attribute: None,
                        default_attribute: None,
                        encoding_attribute: None,
                        encrypt_attribute: None,
                        expression_attribute: None,
                        locale_attribute: None,
                        name_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                            key: AttributeKey {
                                value: "name".to_string(),
                                location: SingleLineLocation::new(22, 2, 4),
                            },
                            value: SpelAttributeValue {
                                equals_location: SingleLineLocation::new(26, 2, 1),
                                opening_quote_location: SingleLineLocation::new(27, 2, 1),
                                spel: SpelAst::Object(SpelResult::Valid(Object::Name(Word {
                                    fragments: vec![WordFragment::String(StringLiteral {
                                        content: "_class".to_string(),
                                        location: spel::ast::Location::VariableLength {
                                            char: 0,
                                            line: 0,
                                            length: 6,
                                        },
                                    })],
                                }))),
                                closing_quote_location: SingleLineLocation::new(34, 2, 1),
                            },
                        })),
                        text_attribute: None,
                        body: None,
                        close_location: SingleLineLocation::new(35, 2, 2),
                    }))),
                    HtmlAttributeValueFragment::Plain(" centered".to_string()),
                ]),
                closing_quote_location: SingleLineLocation::new(46, 2, 1),
            }),
        });
        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser
            .set_language(&tree_sitter_spml::language())
            .map_err(Error::from)?;
        let ts_tree = ts_parser
            .parse(&document, None)
            .ok_or_else(|| anyhow::anyhow!("treesitter parsing failed"))?;
        let mut parser = TreeParser::new(ts_tree.walk(), &document);
        let _header = parser.parse_header()?;
        parser.goto(&NodeMovement::FirstChild);
        parser.goto(&NodeMovement::NextSibling);
        let html = AttributeParser::html(&mut parser)?;
        assert_eq!(html, expected);
        return Ok(());
    }

    #[test]
    pub fn test_tag_in_html_option_attribute() -> Result<()> {
        let document = String::from(concat!(
            "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
            "%>\n",
            "<p class=\"<sp:print name=\"_class\"/> centered\">",
        ));
        let expected = ParsedHtml::Valid(HtmlNode {
            open_location: SingleLineLocation::new(0, 2, 2),
            name: "p".to_string(),
            attributes: vec![ParsedAttribute::Valid(HtmlAttribute {
                key: AttributeKey {
                    value: "class".to_string(),
                    location: SingleLineLocation::new(3, 2, 5),
                },
                value: Some(HtmlAttributeValue {
                    equals_location: SingleLineLocation::new(8, 2, 1),
                    opening_quote_location: SingleLineLocation::new(9, 2, 1),
                    content: HtmlAttributeValueContent::Fragmented(vec![
                        HtmlAttributeValueFragment::Tag(ParsedTag::Valid(SpmlTag::SpPrint(
                            SpPrint {
                                open_location: SingleLineLocation::new(10, 2, 9),
                                arg_attribute: None,
                                condition_attribute: None,
                                convert_attribute: None,
                                cryptkey_attribute: None,
                                dateformat_attribute: None,
                                decimalformat_attribute: None,
                                decoding_attribute: None,
                                decrypt_attribute: None,
                                default_attribute: None,
                                encoding_attribute: None,
                                encrypt_attribute: None,
                                expression_attribute: None,
                                locale_attribute: None,
                                name_attribute: Some(ParsedAttribute::Valid(SpelAttribute {
                                    key: AttributeKey {
                                        value: "name".to_string(),
                                        location: SingleLineLocation::new(20, 2, 4),
                                    },
                                    value: SpelAttributeValue {
                                        equals_location: SingleLineLocation::new(24, 2, 1),
                                        opening_quote_location: SingleLineLocation::new(25, 2, 1),
                                        spel: SpelAst::Object(SpelResult::Valid(Object::Name(
                                            Word {
                                                fragments: vec![WordFragment::String(
                                                    StringLiteral {
                                                        content: "_class".to_string(),
                                                        location:
                                                            spel::ast::Location::VariableLength {
                                                                char: 0,
                                                                line: 0,
                                                                length: 6,
                                                            },
                                                    },
                                                )],
                                            },
                                        ))),
                                        closing_quote_location: SingleLineLocation::new(32, 2, 1),
                                    },
                                })),
                                text_attribute: None,
                                body: None,
                                close_location: SingleLineLocation::new(33, 2, 2),
                            },
                        ))),
                        HtmlAttributeValueFragment::Plain(" centered".to_string()),
                    ]),
                    closing_quote_location: SingleLineLocation::new(44, 2, 1),
                }),
            })],
            body: None,
            close_location: SingleLineLocation::new(45, 2, 1),
        });
        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser
            .set_language(&tree_sitter_spml::language())
            .map_err(Error::from)?;
        let ts_tree = ts_parser
            .parse(&document, None)
            .ok_or_else(|| anyhow::anyhow!("treesitter parsing failed"))?;
        let mut parser = TreeParser::new(ts_tree.walk(), &document);
        let _header = parser.parse_header()?;
        log::info!("kind: {}", parser.cursor.node().kind());
        let html = parser.parse_html_option_tag()?;
        assert_eq!(html, expected);
        return Ok(());
    }
}
