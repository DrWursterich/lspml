use anyhow::Result;
use lsp_types::Position;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tree {
    header: Header,
    tags: Vec<Tag>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Header {
    java_header: PageHeader,
    taglib_imports: Vec<TagLibImport>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PageHeader {
    open_bracket: Location,
    page: Location,
    language: Option<Attribute>,
    page_encoding: Option<Attribute>,
    content_type: Option<Attribute>,
    imports: Vec<Attribute>,
    close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagLibImport {
    open_bracket: Location,
    taglib: Location,
    origin: TagLibOrigin,
    prefix: Attribute,
    close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagLibOrigin {
    Uri(Attribute),
    TagDir(Attribute),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagBody {
    open_location: Location,
    tags: Vec<Tag>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tag {
    // TODO: This is missing text and comments (shouldn't be called Tags then)
    SpAttribute(SpAttribute),
    SpBarcode(SpBarcode),
    SpBreak(SpBreak),
    SpCalendarsheet(SpCalendarsheet),
    SpCheckbox(SpCheckbox),
    SpCode(SpCode),
    SpCollection(SpCollection),
    SpCondition(SpCondition),
    SpDiff(SpDiff),
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
    SptEmail2IMG(SptEmail2IMG),
    SptEncryptemail(SptEncryptemail),
    SptEscapeemail(SptEscapeemail),
    SptFormsolutions(SptFormsolutions),
    SptId2URL(SptId2URL),
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpAttribute {
    open_location: Location,
    condition_attribute: Option<Attribute>,
    default_attribute: Option<Attribute>,
    expression_attribute: Option<Attribute>,
    locale_attribute: Option<Attribute>,
    name_attribute: Option<Attribute>,
    object_attribute: Option<Attribute>,
    value_attribute: Option<Attribute>,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpBarcode {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpBreak {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpCalendarsheet {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpCheckbox {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpCode {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpCollection {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpCondition {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpDiff {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpError {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpExpire {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpFilter {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpFor {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpForm {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpHidden {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpIf {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpInclude {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpIo {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpIterator {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpJson {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpLinkedinformation {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpLinktree {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpLivetree {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpLog {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpLogin {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpLoop {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpMap {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpOption {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpPassword {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpPrint {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpQuerytree {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpRadio {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpRange {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpReturn {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpSass {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpScaleimage {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpScope {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpSearch {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpSelect {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpSet {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpSort {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpSubinformation {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpTagbody {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpText {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpTextarea {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpTextimage {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpThrow {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpToggle {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpUpload {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpUrl {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpWarning {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpWorklist {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpZip {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptCounter {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptDate {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptDiff {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptEmail2IMG {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptEncryptemail {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptEscapeemail {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptFormsolutions {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptId2URL {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptIlink {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptImageeditor {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptImp {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptIterator {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptLink {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptNumber {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptPersonalization {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptPrehtml {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptSmarteditor {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptSpml {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptText {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptTextarea {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptTimestamp {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptTinymce {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptUpdown {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptUpload {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SptWorklist {
    open_location: Location,
    body: Option<TagBody>,
    close_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Attribute {
    key_location: Location,
    equals_location: Location,
    opening_quote_location: Location,
    value: String,
    closing_quote_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Location {
    char: usize,
    line: usize,
    length: usize,
}

impl Location {
    pub(crate) fn new(char: usize, line: usize, length: usize) -> Self {
        return Location { char, line, length };
    }
}

struct TreeParser<'a, 'b> {
    cursor: tree_sitter::TreeCursor<'b>,
    text_bytes: &'a [u8],
}

impl TreeParser<'_, '_> {
    pub(crate) fn new<'a, 'b>(
        cursor: tree_sitter::TreeCursor<'b>,
        text: &'a String,
    ) -> TreeParser<'a, 'b> {
        return TreeParser {
            cursor,
            text_bytes: text.as_bytes(),
        };
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
        let mut java_header = None;
        let mut taglib_imports = Vec::new();
        loop {
            let header_node = self.cursor.node();
            match header_node.kind() {
                "page_header" => match java_header {
                    // technically jsp files CAN have multiple "page" headers...
                    Some(_) => return Err(anyhow::anyhow!("found multiple java headers")),
                    None => java_header = Some(self.parse_page_header()?),
                },
                "taglib_header" => taglib_imports.push(self.parse_taglib_header()?),
                "comment" | "xml_comment" => (),
                _ => break,
            }
            if !self.cursor.goto_next_sibling() {
                // document contains nothing but the header
                break;
            }
        }
        return match java_header {
            Some(java_header) => Ok(Header {
                java_header,
                taglib_imports,
            }),
            None => Err(anyhow::anyhow!("document has no java header")),
        };
    }

    fn parse_page_header(&mut self) -> Result<PageHeader> {
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("java header is empty"));
        }
        let open_bracket = node_location(self.cursor.node());

        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let page = node_location(self.cursor.node());

        let mut content_type = None;
        let mut language = None;
        let mut page_encoding = None;
        let mut imports = Vec::new();
        loop {
            if !self.cursor.goto_next_sibling() {
                return Err(anyhow::anyhow!("java header is unclosed"));
            }
            let node = self.cursor.node();
            match node.kind() {
                "import_attribute" => imports.push(self.parse_attribute()?.1),
                "contentType_attribute" => content_type = Some(self.parse_attribute()?.1),
                "language_attribute" => language = Some(self.parse_attribute()?.1),
                "pageEncoding_attribute" => page_encoding = Some(self.parse_attribute()?.1),
                "header_close" => break,
                kind => return Err(anyhow::anyhow!("unexpected {}", kind)),
            }
        }
        let close_bracket = node_location(self.cursor.node());
        self.cursor.goto_parent();
        return Ok(PageHeader {
            open_bracket,
            page,
            language,
            page_encoding,
            content_type,
            imports,
            close_bracket,
        });
    }

    fn parse_taglib_header(&mut self) -> Result<TagLibImport> {
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("java header is empty"));
        }
        let open_bracket = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let taglib = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"language\" attribute"
            ));
        }
        // TODO: attribute order should not matter
        let (name, attribute) = self.parse_attribute()?;
        let origin = match name.as_str() {
            "uri" => TagLibOrigin::Uri(attribute),
            "tagdir" => TagLibOrigin::TagDir(attribute),
            name => {
                return Err(anyhow::anyhow!(
                    "unexpected \"{}\" attribute in taglib header",
                    name
                ))
            }
        };
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"pageEncoding\" attribute"
            ));
        }
        let (_, prefix) = self.parse_attribute()?;
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let close_bracket = node_location(self.cursor.node());
        self.cursor.goto_parent();
        return Ok(TagLibImport {
            open_bracket,
            taglib,
            origin,
            prefix,
            close_bracket,
        });
    }

    fn parse_tags(&mut self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();
        loop {
            if !self.cursor.goto_next_sibling() {
                break;
            }
            let node = self.cursor.node();
            match node.kind() {
                "comment" | "xml_comment" | "text" | "xml_entity" => (),
                "argument_tag" => tags.push(self.parse_sp_attribute()?),
                // TODO: I don't want to handle each tag individually.
                //       Can I write a macro for that or something?
                _ => break,
            };
        }
        return Ok(tags);
    }

    fn parse_sp_attribute(&mut self) -> Result<Tag> {
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("tag is empty"));
        }
        let open_location = node_location(self.cursor.node());
        let mut condition_attribute = None;
        let mut default_attribute = None;
        let mut expression_attribute = None;
        let mut locale_attribute = None;
        let mut name_attribute = None;
        let mut object_attribute = None;
        let mut value_attribute = None;
        let mut body = None;
        loop {
            if !self.cursor.goto_next_sibling() {
                return Err(anyhow::anyhow!("sp:attribute tag is unclosed"));
            }
            let node = self.cursor.node();
            match node.kind() {
                "comment" | "xml_comment" => (),
                "condition_attribute" => condition_attribute = Some(self.parse_attribute()?.1),
                "default_attribute" => default_attribute = Some(self.parse_attribute()?.1),
                "expression_attribute" => expression_attribute = Some(self.parse_attribute()?.1),
                "locale_attribute" => locale_attribute = Some(self.parse_attribute()?.1),
                "name_attribute" => name_attribute = Some(self.parse_attribute()?.1),
                "object_attribute" => object_attribute = Some(self.parse_attribute()?.1),
                "value_attribute" => value_attribute = Some(self.parse_attribute()?.1),
                "self_closing_tag_end" => break,
                ">" => body = Some(self.parse_tag_body()?),
                _ => (),
            };
        }
        let close_location = node_location(self.cursor.node());
        return Ok(Tag::SpAttribute(SpAttribute {
            open_location,
            condition_attribute,
            default_attribute,
            expression_attribute,
            locale_attribute,
            name_attribute,
            object_attribute,
            value_attribute,
            body,
            close_location,
        }));
    }

    fn parse_tag_body(&mut self) -> Result<TagBody> {
        let open_location = node_location(self.cursor.node());
        let tags = self.parse_tags()?;
        return Ok(TagBody {
            open_location,
            tags,
        });
    }

    fn parse_attribute(&mut self) -> Result<(String, Attribute)> {
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("attribute is empty"));
        }
        let key_node = self.cursor.node();
        let key_location = node_location(key_node);
        let key = key_node.kind().to_string();
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute is missing a value"));
        }
        let equals_location = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute is missing a value"));
        }
        let _string_node = self.cursor.node();
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("attribute value is empty"));
        }
        let opening_quote_location = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute value string is unclosed"));
        }
        let value = self.cursor.node().utf8_text(self.text_bytes)?.to_string();
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute value string is unclosed"));
        }
        let closing_quote_location = node_location(self.cursor.node());
        self.cursor.goto_parent();
        self.cursor.goto_parent();
        let attribute = Attribute {
            key_location,
            equals_location,
            opening_quote_location,
            value,
            closing_quote_location,
        };
        return Ok((key, attribute));
    }
}

fn node_location(node: tree_sitter::Node) -> Location {
    let start = node.start_position();
    return Location {
        char: start.column,
        line: start.row,
        length: node.end_position().column - start.column,
    };
}

impl Tree {
    pub(crate) fn new(ts: tree_sitter::Tree, text: String) -> Result<Self> {
        let parser = &mut TreeParser::new(ts.walk(), &text);
        let header = parser.parse_header()?;
        let tags = parser.parse_tags()?;
        return Ok(Tree { header, tags });
    }
}

// =============================================================================================
// || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF ||
// =============================================================================================

pub(crate) fn find_current_node<'tree>(
    tree: &'tree tree_sitter::Tree,
    position: Position,
) -> Option<tree_sitter::Node<'tree>> {
    let trigger_point =
        tree_sitter::Point::new(position.line as usize, position.character as usize);
    let mut cursor = tree.root_node().walk();
    loop {
        let node = cursor.node();
        if match node.end_position() <= trigger_point {
            true => !cursor.goto_next_sibling() || cursor.node().start_position() > trigger_point,
            false => !cursor.goto_first_child(),
        } {
            log::debug!("current node: {:?}", node);
            return Some(node);
        }
    }
}

pub(crate) fn attribute_name_of<'a>(
    attribute: tree_sitter::Node<'_>,
    source: &'a str,
) -> Option<&'a str> {
    return attribute
        .child(0)
        .and_then(|node| node.utf8_text(source.as_bytes()).ok());
}

pub(crate) fn attribute_value_of<'a>(
    attribute: tree_sitter::Node<'_>,
    source: &'a str,
) -> Option<&'a str> {
    return attribute
        .child(2)
        .and_then(|node| node.child(1))
        .and_then(|node| node.utf8_text(source.as_bytes()).ok());
}

pub(crate) fn attribute_name_and_value_of<'a>(
    attribute: tree_sitter::Node<'_>,
    source: &'a str,
) -> Option<(&'a str, &'a str)> {
    return attribute
        .child(0)
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(|name| {
            (
                name,
                attribute
                    .child(2)
                    .and_then(|node| node.child(1))
                    .filter(|node| node.kind() == "string_content")
                    .and_then(|node| node.utf8_text(source.as_bytes()).ok())
                    .unwrap_or(""),
            )
        });
}

// ============================================================================================
// || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS ||
// ============================================================================================

#[cfg(test)]
mod tests {
    use crate::parser::{Attribute, Header, Location, PageHeader, TagLibImport, TagLibOrigin};

    use super::TreeParser;
    use anyhow::{Error, Result};

    #[test]
    fn test_parse_header() -> Result<()> {
        let document = String::from(concat!(
            "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
            "%><%@ taglib uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n",
            "%><%@ taglib tagdir=\"/WEB-INF/tags/spt\" prefix=\"spt\"\n",
            "%>\n"
        ));
        let expected = Header {
            java_header: PageHeader {
                open_bracket: Location::new(0, 0, 3),
                page: Location::new(4, 0, 4),
                language: Some(Attribute {
                    key_location: Location::new(9, 0, 8),
                    equals_location: Location::new(17, 0, 1),
                    opening_quote_location: Location::new(18, 0, 1),
                    value: "java".to_string(),
                    closing_quote_location: Location::new(23, 0, 1),
                }),
                page_encoding: Some(Attribute {
                    key_location: Location::new(25, 0, 12),
                    equals_location: Location::new(37, 0, 1),
                    opening_quote_location: Location::new(38, 0, 1),
                    value: "UTF-8".to_string(),
                    closing_quote_location: Location::new(44, 0, 1),
                }),
                content_type: Some(Attribute {
                    key_location: Location::new(46, 0, 11),
                    equals_location: Location::new(57, 0, 1),
                    opening_quote_location: Location::new(58, 0, 1),
                    value: "text/html; charset=UTF-8".to_string(),
                    closing_quote_location: Location::new(83, 0, 1),
                }),
                imports: vec![],
                close_bracket: Location::new(0, 1, 2),
            },
            taglib_imports: vec![
                TagLibImport {
                    open_bracket: Location {
                        char: 2,
                        line: 1,
                        length: 3,
                    },
                    taglib: Location {
                        char: 6,
                        line: 1,
                        length: 6,
                    },
                    origin: TagLibOrigin::Uri(Attribute {
                        key_location: Location {
                            char: 13,
                            line: 1,
                            length: 3,
                        },
                        equals_location: Location {
                            char: 16,
                            line: 1,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 17,
                            line: 1,
                            length: 1,
                        },
                        value: "http://www.sitepark.com/taglibs/core".to_string(),
                        closing_quote_location: Location {
                            char: 54,
                            line: 1,
                            length: 1,
                        },
                    }),
                    prefix: Attribute {
                        key_location: Location {
                            char: 56,
                            line: 1,
                            length: 6,
                        },
                        equals_location: Location {
                            char: 62,
                            line: 1,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 63,
                            line: 1,
                            length: 1,
                        },
                        value: "sp".to_string(),
                        closing_quote_location: Location {
                            char: 66,
                            line: 1,
                            length: 1,
                        },
                    },
                    close_bracket: Location {
                        char: 0,
                        line: 2,
                        length: 2,
                    },
                },
                TagLibImport {
                    open_bracket: Location {
                        char: 2,
                        line: 2,
                        length: 3,
                    },
                    taglib: Location {
                        char: 6,
                        line: 2,
                        length: 6,
                    },
                    origin: TagLibOrigin::TagDir(Attribute {
                        key_location: Location {
                            char: 13,
                            line: 2,
                            length: 6,
                        },
                        equals_location: Location {
                            char: 19,
                            line: 2,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 20,
                            line: 2,
                            length: 1,
                        },
                        value: "/WEB-INF/tags/spt".to_string(),
                        closing_quote_location: Location {
                            char: 38,
                            line: 2,
                            length: 1,
                        },
                    }),
                    prefix: Attribute {
                        key_location: Location {
                            char: 40,
                            line: 2,
                            length: 6,
                        },
                        equals_location: Location {
                            char: 46,
                            line: 2,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 47,
                            line: 2,
                            length: 1,
                        },
                        value: "spt".to_string(),
                        closing_quote_location: Location {
                            char: 51,
                            line: 2,
                            length: 1,
                        },
                    },
                    close_bracket: Location {
                        char: 0,
                        line: 3,
                        length: 2,
                    },
                },
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
}
