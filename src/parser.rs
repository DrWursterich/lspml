use anyhow::Result;
use lsp_types::Position;

pub use derive::ParsableTag;

use crate::{
    grammar::{TagAttributeType, TagDefinition},
    spel::{
        self,
        ast::{SpelAst, SpelResult},
    },
};

pub(crate) trait ParsableTag {
    fn parse(parser: &mut TreeParser) -> Result<Self>
    where
        Self: Sized;
}

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
    language: Option<PlainAttribute>,
    page_encoding: Option<PlainAttribute>,
    content_type: Option<PlainAttribute>,
    imports: Vec<PlainAttribute>,
    close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagLibImport {
    open_bracket: Location,
    taglib: Location,
    origin: TagLibOrigin,
    prefix: PlainAttribute,
    close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagLibOrigin {
    Uri(PlainAttribute),
    TagDir(PlainAttribute),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagBody {
    open_location: Location,
    tags: Vec<Tag>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tag {
    // TODO: This is missing text and comments (shouldn't be called Tags then)
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

macro_rules! tag_struct {
    (#[$definition:expr] $name:ident { $( $param:ident ),* $(,)* }) => {
        #[derive(Clone, Debug, PartialEq, ParsableTag)]
        #[tag_definition($definition)]
        pub struct $name {
            pub open_location: Location,
            $(pub $param: Option<SpelAttribute>,)*
            pub body: Option<TagBody>,
            pub close_location: Location,
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
    #[TagDefinition::SP_Break]
    SpBreak { }
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
    SpCode { }
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
    SpCondition { }
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
    SpElse { }
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
    SpError {
        code_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_EXPIRE]
    SpExpire {
        date_attribute,
    }
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
    SpLinkedinformation {
    }
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
    SpLog {
        level_attribute,
    }
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
    SpPassword { }
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
    SpQuerytree { }
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
    SpScope {
        scope_attribute,
    }
);

tag_struct!(
    #[TagDefinition::SP_SEARCH]
    SpSearch { }
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
    SpTagbody { }
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
    SpThrow { }
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
    SpWarning {
        code_attribute,
    }
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
    SpZip { }
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
    SptSpml {
        api_attribute,
    }
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
    SptTimestamp {
        connect_attribute,
    }
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Attribute {
    Plain(PlainAttribute),
    Spel(SpelAttribute),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlainAttribute {
    key_location: Location,
    equals_location: Location,
    opening_quote_location: Location,
    value: String,
    closing_quote_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpelAttribute {
    key_location: Location,
    equals_location: Location,
    opening_quote_location: Location,
    spel: SpelAst,
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

pub(crate) struct TreeParser<'a, 'b> {
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
                "import_attribute" => imports.push(self.parse_plain_attribute()?.1),
                "contentType_attribute" => content_type = Some(self.parse_plain_attribute()?.1),
                "language_attribute" => language = Some(self.parse_plain_attribute()?.1),
                "pageEncoding_attribute" => page_encoding = Some(self.parse_plain_attribute()?.1),
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
        let (name, attribute) = self.parse_plain_attribute()?;
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
        let (_, prefix) = self.parse_plain_attribute()?;
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
            let node = self.cursor.node();
            match node.kind() {
                "comment" | "xml_comment" | "text" | "xml_entity" => (),
                "attribute_tag" => tags.push(SpAttribute::parse(self).map(Tag::SpAttribute)?),
                "argument_tag" => tags.push(SpArgument::parse(self).map(Tag::SpArgument)?),
                "barcode_tag" => tags.push(SpBarcode::parse(self).map(Tag::SpBarcode)?),
                "break_tag" => tags.push(SpBreak::parse(self).map(Tag::SpBreak)?),
                "calendarsheet_tag" => tags.push(SpCalendarsheet::parse(self).map(Tag::SpCalendarsheet)?),
                "checkbox_tag" => tags.push(SpCheckbox::parse(self).map(Tag::SpCheckbox)?),
                "code_tag" => tags.push(SpCode::parse(self).map(Tag::SpCode)?),
                "collection_tag" => tags.push(SpCollection::parse(self).map(Tag::SpCollection)?),
                "condition_tag" => tags.push(SpCondition::parse(self).map(Tag::SpCondition)?),
                "diff_tag" => tags.push(SpDiff::parse(self).map(Tag::SpDiff)?),
                "else_tag" => tags.push(SpElse::parse(self).map(Tag::SpElse)?),
                "elseIf_tag" => tags.push(SpElseIf::parse(self).map(Tag::SpElseIf)?),
                "error_tag" => tags.push(SpError::parse(self).map(Tag::SpError)?),
                "expire_tag" => tags.push(SpExpire::parse(self).map(Tag::SpExpire)?),
                "filter_tag" => tags.push(SpFilter::parse(self).map(Tag::SpFilter)?),
                "for_tag" => tags.push(SpFor::parse(self).map(Tag::SpFor)?),
                "form_tag" => tags.push(SpForm::parse(self).map(Tag::SpForm)?),
                "hidden_tag" => tags.push(SpHidden::parse(self).map(Tag::SpHidden)?),
                "if_tag" => tags.push(SpIf::parse(self).map(Tag::SpIf)?),
                "include_tag" => tags.push(SpInclude::parse(self).map(Tag::SpInclude)?),
                "io_tag" => tags.push(SpIo::parse(self).map(Tag::SpIo)?),
                "iterator_tag" => tags.push(SpIterator::parse(self).map(Tag::SpIterator)?),
                "json_tag" => tags.push(SpJson::parse(self).map(Tag::SpJson)?),
                "linkedinformation_tag" => tags.push(SpLinkedinformation::parse(self).map(Tag::SpLinkedinformation)?),
                "linktree_tag" => tags.push(SpLinktree::parse(self).map(Tag::SpLinktree)?),
                "livetree_tag" => tags.push(SpLivetree::parse(self).map(Tag::SpLivetree)?),
                "log_tag" => tags.push(SpLog::parse(self).map(Tag::SpLog)?),
                "login_tag" => tags.push(SpLogin::parse(self).map(Tag::SpLogin)?),
                "loop_tag" => tags.push(SpLoop::parse(self).map(Tag::SpLoop)?),
                "map_tag" => tags.push(SpMap::parse(self).map(Tag::SpMap)?),
                "option_tag" => tags.push(SpOption::parse(self).map(Tag::SpOption)?),
                "password_tag" => tags.push(SpPassword::parse(self).map(Tag::SpPassword)?),
                "print_tag" => tags.push(SpPrint::parse(self).map(Tag::SpPrint)?),
                "querytree_tag" => tags.push(SpQuerytree::parse(self).map(Tag::SpQuerytree)?),
                "radio_tag" => tags.push(SpRadio::parse(self).map(Tag::SpRadio)?),
                "range_tag" => tags.push(SpRange::parse(self).map(Tag::SpRange)?),
                "return_tag" => tags.push(SpReturn::parse(self).map(Tag::SpReturn)?),
                "sass_tag" => tags.push(SpSass::parse(self).map(Tag::SpSass)?),
                "scaleimage_tag" => tags.push(SpScaleimage::parse(self).map(Tag::SpScaleimage)?),
                "scope_tag" => tags.push(SpScope::parse(self).map(Tag::SpScope)?),
                "search_tag" => tags.push(SpSearch::parse(self).map(Tag::SpSearch)?),
                "select_tag" => tags.push(SpSelect::parse(self).map(Tag::SpSelect)?),
                "set_tag" => tags.push(SpSet::parse(self).map(Tag::SpSet)?),
                "sort_tag" => tags.push(SpSort::parse(self).map(Tag::SpSort)?),
                "subinformation_tag" => tags.push(SpSubinformation::parse(self).map(Tag::SpSubinformation)?),
                "tagbody_tag" => tags.push(SpTagbody::parse(self).map(Tag::SpTagbody)?),
                "text_tag" => tags.push(SpText::parse(self).map(Tag::SpText)?),
                "textarea_tag" => tags.push(SpTextarea::parse(self).map(Tag::SpTextarea)?),
                "textimage_tag" => tags.push(SpTextimage::parse(self).map(Tag::SpTextimage)?),
                "throw_tag" => tags.push(SpThrow::parse(self).map(Tag::SpThrow)?),
                "toggle_tag" => tags.push(SpToggle::parse(self).map(Tag::SpToggle)?),
                "upload_tag" => tags.push(SpUpload::parse(self).map(Tag::SpUpload)?),
                "url_tag" => tags.push(SpUrl::parse(self).map(Tag::SpUrl)?),
                "warning_tag" => tags.push(SpWarning::parse(self).map(Tag::SpWarning)?),
                "worklist_tag" => tags.push(SpWorklist::parse(self).map(Tag::SpWorklist)?),
                "zip_tag" => tags.push(SpZip::parse(self).map(Tag::SpZip)?),
                "spt_counter_tag" => tags.push(SptCounter::parse(self).map(Tag::SptCounter)?),
                "spt_date_tag" => tags.push(SptDate::parse(self).map(Tag::SptDate)?),
                "spt_diff_tag" => tags.push(SptDiff::parse(self).map(Tag::SptDiff)?),
                "spt_email2img_tag" => tags.push(SptEmail2Img::parse(self).map(Tag::SptEmail2Img)?),
                "spt_encryptemail_tag" => tags.push(SptEncryptemail::parse(self).map(Tag::SptEncryptemail)?),
                "spt_escapeemail_tag" => tags.push(SptEscapeemail::parse(self).map(Tag::SptEscapeemail)?),
                "spt_formsolutions_tag" => tags.push(SptFormsolutions::parse(self).map(Tag::SptFormsolutions)?),
                "spt_id2url_tag" => tags.push(SptId2Url::parse(self).map(Tag::SptId2Url)?),
                "spt_ilink_tag" => tags.push(SptIlink::parse(self).map(Tag::SptIlink)?),
                "spt_imageeditor_tag" => tags.push(SptImageeditor::parse(self).map(Tag::SptImageeditor)?),
                "spt_imp_tag" => tags.push(SptImp::parse(self).map(Tag::SptImp)?),
                "spt_iterator_tag" => tags.push(SptIterator::parse(self).map(Tag::SptIterator)?),
                "spt_link_tag" => tags.push(SptLink::parse(self).map(Tag::SptLink)?),
                "spt_number_tag" => tags.push(SptNumber::parse(self).map(Tag::SptNumber)?),
                "spt_personalization_tag" => tags.push(SptPersonalization::parse(self).map(Tag::SptPersonalization)?),
                "spt_prehtml_tag" => tags.push(SptPrehtml::parse(self).map(Tag::SptPrehtml)?),
                "spt_smarteditor_tag" => tags.push(SptSmarteditor::parse(self).map(Tag::SptSmarteditor)?),
                "spt_spml_tag" => tags.push(SptSpml::parse(self).map(Tag::SptSpml)?),
                "spt_text_tag" => tags.push(SptText::parse(self).map(Tag::SptText)?),
                "spt_textarea_tag" => tags.push(SptTextarea::parse(self).map(Tag::SptTextarea)?),
                "spt_timestamp_tag" => tags.push(SptTimestamp::parse(self).map(Tag::SptTimestamp)?),
                "spt_tinymce_tag" => tags.push(SptTinymce::parse(self).map(Tag::SptTinymce)?),
                "spt_updown_tag" => tags.push(SptUpdown::parse(self).map(Tag::SptUpdown)?),
                "spt_upload_tag" => tags.push(SptUpload::parse(self).map(Tag::SptUpload)?),
                "spt_worklist_tag" => tags.push(SptWorklist::parse(self).map(Tag::SptWorklist)?),
                kind => {
                    log::warn!("parse_tags break on {}", kind);
                    break;
                }
            };
            if !self.cursor.goto_next_sibling() {
                break;
            }
        }
        return Ok(tags);
    }

    fn parse_tag_body(&mut self) -> Result<TagBody> {
        let open_location = node_location(self.cursor.node());
        let tags = self.parse_tags()?;
        return Ok(TagBody {
            open_location,
            tags,
        });
    }

    fn parse_plain_attribute(&mut self) -> Result<(String, PlainAttribute)> {
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
        let attribute = PlainAttribute {
            key_location,
            equals_location,
            opening_quote_location,
            value,
            closing_quote_location,
        };
        return Ok((key, attribute));
    }

    fn parse_spel_attribute(
        &mut self,
        r#type: &TagAttributeType,
    ) -> Result<(String, SpelAttribute)> {
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
        let text = self.cursor.node().utf8_text(self.text_bytes)?;

        let parser = &mut spel::parser::Parser::new(text);
        let spel = match r#type {
            TagAttributeType::Comparable => match parser.parse_comparable() {
                Ok(result) => SpelAst::Comparable(SpelResult::Valid(result)),
                // workaround as comparables as attribute values do accept strings (without quotes)
                // but comparables in actuall comparissons do not.
                Err(err) => match spel::parser::Parser::new(text).parse_text() {
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

        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute value string is unclosed"));
        }
        let closing_quote_location = node_location(self.cursor.node());
        self.cursor.goto_parent();
        self.cursor.goto_parent();
        let attribute = SpelAttribute {
            key_location,
            equals_location,
            opening_quote_location,
            spel,
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
    use crate::{
        parser::{
            Header, Location, PageHeader, PlainAttribute, SpBarcode, SpelAttribute, Tag,
            TagLibImport, TagLibOrigin,
        },
        spel::{
            self,
            ast::{Identifier, SpelAst, SpelResult, StringLiteral, Word, WordFragment},
        },
    };

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
                language: Some(PlainAttribute {
                    key_location: Location::new(9, 0, 8),
                    equals_location: Location::new(17, 0, 1),
                    opening_quote_location: Location::new(18, 0, 1),
                    value: "java".to_string(),
                    closing_quote_location: Location::new(23, 0, 1),
                }),
                page_encoding: Some(PlainAttribute {
                    key_location: Location::new(25, 0, 12),
                    equals_location: Location::new(37, 0, 1),
                    opening_quote_location: Location::new(38, 0, 1),
                    value: "UTF-8".to_string(),
                    closing_quote_location: Location::new(44, 0, 1),
                }),
                content_type: Some(PlainAttribute {
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
                    origin: TagLibOrigin::Uri(PlainAttribute {
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
                    prefix: PlainAttribute {
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
                    origin: TagLibOrigin::TagDir(PlainAttribute {
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
                    prefix: PlainAttribute {
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

    #[test]
    fn test_parse_tags() -> Result<()> {
        let document = String::from(concat!(
            "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
            "%>\n",
            "<sp:barcode name=\"_testName\" text=\"some text\" scope=\"page\"/>\n",
        ));
        let expected = vec![Tag::SpBarcode(SpBarcode {
            open_location: Location {
                char: 0,
                line: 2,
                length: 11,
            },
            height_attribute: None,
            locale_attribute: None,
            name_attribute: Some(SpelAttribute {
                key_location: Location {
                    char: 12,
                    line: 2,
                    length: 4,
                },
                equals_location: Location {
                    char: 16,
                    line: 2,
                    length: 1,
                },
                opening_quote_location: Location {
                    char: 17,
                    line: 2,
                    length: 1,
                },
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
                closing_quote_location: Location {
                    char: 27,
                    line: 2,
                    length: 1,
                },
            }),
            scope_attribute: Some(SpelAttribute {
                key_location: Location {
                    char: 46,
                    line: 2,
                    length: 5,
                },
                equals_location: Location {
                    char: 51,
                    line: 2,
                    length: 1,
                },
                opening_quote_location: Location {
                    char: 52,
                    line: 2,
                    length: 1,
                },
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
                closing_quote_location: Location {
                    char: 57,
                    line: 2,
                    length: 1,
                },
            }),
            text_attribute: Some(SpelAttribute {
                key_location: Location {
                    char: 29,
                    line: 2,
                    length: 4,
                },
                equals_location: Location {
                    char: 33,
                    line: 2,
                    length: 1,
                },
                opening_quote_location: Location {
                    char: 34,
                    line: 2,
                    length: 1,
                },
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
                closing_quote_location: Location {
                    char: 44,
                    line: 2,
                    length: 1,
                },
            }),
            type_attribute: None,
            body: None,
            close_location: Location {
                char: 58,
                line: 2,
                length: 2,
            },
        })];
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
}
