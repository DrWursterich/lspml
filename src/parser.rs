#![allow(non_snake_case)]

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
    fn parse(parser: &mut TreeParser) -> Result<Self>
    where
        Self: Sized;

    fn definition(&self) -> TagDefinition;

    fn open_location(&self) -> &Location;

    fn close_location(&self) -> &Location;

    fn body(&self) -> &Option<TagBody>;

    fn spel_attributes(&self) -> Vec<(&str, &SpelAttribute)>;

    fn spel_attribute(&self, name: &str) -> Option<&SpelAttribute>;
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tree {
    pub(crate) header: Header,
    pub(crate) nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Header {
    pub(crate) java_headers: Vec<PageHeader>,
    pub(crate) taglib_imports: Vec<TagLibImport>,
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
    pub(crate) open_location: Location,
    pub(crate) nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) enum Node {
    Tag(Tag),
    Html(HtmlNode),
    Text(TextNode),
    Error(ErrorNode),
}

#[derive(Clone, Debug, PartialEq, DocumentNode)]
pub(crate) struct HtmlNode {
    pub(crate) open_location: Location,
    pub(crate) name: String,
    pub(crate) attributes: Vec<PlainAttribute>,
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
pub(crate) enum Tag {
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
        #[derive(Clone, Debug, PartialEq, DocumentNode, ParsableTag)]
        #[tag_definition($definition)]
        pub(crate) struct $name {
            pub(crate) open_location: Location,
            $(pub(crate) $param: Option<SpelAttribute>,)*
            pub(crate) body: Option<TagBody>,
            pub(crate) close_location: Location,
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

pub(crate) trait Attribute {
    // fn key_location(&self) -> &Location;

    // fn equals_location(&self) -> &Location;

    fn opening_quote_location(&self) -> &Location;

    fn closing_quote_location(&self) -> &Location;
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlainAttribute {
    pub(crate) key_location: Location,
    pub(crate) equals_location: Location,
    pub(crate) opening_quote_location: Location,
    pub(crate) value: String,
    pub(crate) closing_quote_location: Location,
}

impl Attribute for PlainAttribute {
    // fn key_location(&self) -> &Location {
    //     return &self.key_location;
    // }

    // fn equals_location(&self) -> &Location {
    //     return &self.equals_location;
    // }

    fn opening_quote_location(&self) -> &Location {
        return &self.opening_quote_location;
    }

    fn closing_quote_location(&self) -> &Location {
        return &self.closing_quote_location;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpelAttribute {
    pub(crate) key_location: Location,
    pub(crate) equals_location: Location,
    pub(crate) opening_quote_location: Location,
    pub(crate) spel: SpelAst,
    pub(crate) closing_quote_location: Location,
}

impl Attribute for SpelAttribute {
    // fn key_location(&self) -> &Location {
    //     return &self.key_location;
    // }

    // fn equals_location(&self) -> &Location {
    //     return &self.equals_location;
    // }

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

    pub(crate) fn contains(&self, position: Position) -> bool {
        return self.line == position.line as usize
            && self.char <= position.character as usize
            && self.char + self.length > position.character as usize;
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
                        .map(Tag::SpAttribute)
                        .map(Node::Tag)?,
                ),
                "argument_tag" => tags.push(
                    SpArgument::parse(self)
                        .map(Tag::SpArgument)
                        .map(Node::Tag)?,
                ),
                "barcode_tag" => {
                    tags.push(SpBarcode::parse(self).map(Tag::SpBarcode).map(Node::Tag)?)
                }
                "break_tag" => tags.push(SpBreak::parse(self).map(Tag::SpBreak).map(Node::Tag)?),
                "calendarsheet_tag" => tags.push(
                    SpCalendarsheet::parse(self)
                        .map(Tag::SpCalendarsheet)
                        .map(Node::Tag)?,
                ),
                "checkbox_tag" => tags.push(
                    SpCheckbox::parse(self)
                        .map(Tag::SpCheckbox)
                        .map(Node::Tag)?,
                ),
                "code_tag" => tags.push(SpCode::parse(self).map(Tag::SpCode).map(Node::Tag)?),
                "collection_tag" => tags.push(
                    SpCollection::parse(self)
                        .map(Tag::SpCollection)
                        .map(Node::Tag)?,
                ),
                "condition_tag" => tags.push(
                    SpCondition::parse(self)
                        .map(Tag::SpCondition)
                        .map(Node::Tag)?,
                ),
                "diff_tag" => tags.push(SpDiff::parse(self).map(Tag::SpDiff).map(Node::Tag)?),
                "else_tag" => tags.push(SpElse::parse(self).map(Tag::SpElse).map(Node::Tag)?),
                "elseIf_tag" => tags.push(SpElseIf::parse(self).map(Tag::SpElseIf).map(Node::Tag)?),
                "error_tag" => tags.push(SpError::parse(self).map(Tag::SpError).map(Node::Tag)?),
                "expire_tag" => tags.push(SpExpire::parse(self).map(Tag::SpExpire).map(Node::Tag)?),
                "filter_tag" => tags.push(SpFilter::parse(self).map(Tag::SpFilter).map(Node::Tag)?),
                "for_tag" => tags.push(SpFor::parse(self).map(Tag::SpFor).map(Node::Tag)?),
                "form_tag" => tags.push(SpForm::parse(self).map(Tag::SpForm).map(Node::Tag)?),
                "hidden_tag" => tags.push(SpHidden::parse(self).map(Tag::SpHidden).map(Node::Tag)?),
                "if_tag" => tags.push(SpIf::parse(self).map(Tag::SpIf).map(Node::Tag)?),
                "include_tag" => {
                    tags.push(SpInclude::parse(self).map(Tag::SpInclude).map(Node::Tag)?)
                }
                "io_tag" => tags.push(SpIo::parse(self).map(Tag::SpIo).map(Node::Tag)?),
                "iterator_tag" => tags.push(
                    SpIterator::parse(self)
                        .map(Tag::SpIterator)
                        .map(Node::Tag)?,
                ),
                "json_tag" => tags.push(SpJson::parse(self).map(Tag::SpJson).map(Node::Tag)?),
                "linkedinformation_tag" => tags.push(
                    SpLinkedinformation::parse(self)
                        .map(Tag::SpLinkedinformation)
                        .map(Node::Tag)?,
                ),
                "linktree_tag" => tags.push(
                    SpLinktree::parse(self)
                        .map(Tag::SpLinktree)
                        .map(Node::Tag)?,
                ),
                "livetree_tag" => tags.push(
                    SpLivetree::parse(self)
                        .map(Tag::SpLivetree)
                        .map(Node::Tag)?,
                ),
                "log_tag" => tags.push(SpLog::parse(self).map(Tag::SpLog).map(Node::Tag)?),
                "login_tag" => tags.push(SpLogin::parse(self).map(Tag::SpLogin).map(Node::Tag)?),
                "loop_tag" => tags.push(SpLoop::parse(self).map(Tag::SpLoop).map(Node::Tag)?),
                "map_tag" => tags.push(SpMap::parse(self).map(Tag::SpMap).map(Node::Tag)?),
                "option_tag" => tags.push(SpOption::parse(self).map(Tag::SpOption).map(Node::Tag)?),
                "password_tag" => tags.push(
                    SpPassword::parse(self)
                        .map(Tag::SpPassword)
                        .map(Node::Tag)?,
                ),
                "print_tag" => tags.push(SpPrint::parse(self).map(Tag::SpPrint).map(Node::Tag)?),
                "querytree_tag" => tags.push(
                    SpQuerytree::parse(self)
                        .map(Tag::SpQuerytree)
                        .map(Node::Tag)?,
                ),
                "radio_tag" => tags.push(SpRadio::parse(self).map(Tag::SpRadio).map(Node::Tag)?),
                "range_tag" => tags.push(SpRange::parse(self).map(Tag::SpRange).map(Node::Tag)?),
                "return_tag" => tags.push(SpReturn::parse(self).map(Tag::SpReturn).map(Node::Tag)?),
                "sass_tag" => tags.push(SpSass::parse(self).map(Tag::SpSass).map(Node::Tag)?),
                "scaleimage_tag" => tags.push(
                    SpScaleimage::parse(self)
                        .map(Tag::SpScaleimage)
                        .map(Node::Tag)?,
                ),
                "scope_tag" => tags.push(SpScope::parse(self).map(Tag::SpScope).map(Node::Tag)?),
                "search_tag" => tags.push(SpSearch::parse(self).map(Tag::SpSearch).map(Node::Tag)?),
                "select_tag" => tags.push(SpSelect::parse(self).map(Tag::SpSelect).map(Node::Tag)?),
                "set_tag" => tags.push(SpSet::parse(self).map(Tag::SpSet).map(Node::Tag)?),
                "sort_tag" => tags.push(SpSort::parse(self).map(Tag::SpSort).map(Node::Tag)?),
                "subinformation_tag" => tags.push(
                    SpSubinformation::parse(self)
                        .map(Tag::SpSubinformation)
                        .map(Node::Tag)?,
                ),
                "tagbody_tag" => {
                    tags.push(SpTagbody::parse(self).map(Tag::SpTagbody).map(Node::Tag)?)
                }
                "text_tag" => tags.push(SpText::parse(self).map(Tag::SpText).map(Node::Tag)?),
                "textarea_tag" => tags.push(
                    SpTextarea::parse(self)
                        .map(Tag::SpTextarea)
                        .map(Node::Tag)?,
                ),
                "textimage_tag" => tags.push(
                    SpTextimage::parse(self)
                        .map(Tag::SpTextimage)
                        .map(Node::Tag)?,
                ),
                "throw_tag" => tags.push(SpThrow::parse(self).map(Tag::SpThrow).map(Node::Tag)?),
                "toggle_tag" => tags.push(SpToggle::parse(self).map(Tag::SpToggle).map(Node::Tag)?),
                "upload_tag" => tags.push(SpUpload::parse(self).map(Tag::SpUpload).map(Node::Tag)?),
                "url_tag" => tags.push(SpUrl::parse(self).map(Tag::SpUrl).map(Node::Tag)?),
                "warning_tag" => {
                    tags.push(SpWarning::parse(self).map(Tag::SpWarning).map(Node::Tag)?)
                }
                "worklist_tag" => tags.push(
                    SpWorklist::parse(self)
                        .map(Tag::SpWorklist)
                        .map(Node::Tag)?,
                ),
                "zip_tag" => tags.push(SpZip::parse(self).map(Tag::SpZip).map(Node::Tag)?),
                "spt_counter_tag" => tags.push(
                    SptCounter::parse(self)
                        .map(Tag::SptCounter)
                        .map(Node::Tag)?,
                ),
                "spt_date_tag" => tags.push(SptDate::parse(self).map(Tag::SptDate).map(Node::Tag)?),
                "spt_diff_tag" => tags.push(SptDiff::parse(self).map(Tag::SptDiff).map(Node::Tag)?),
                "spt_email2img_tag" => tags.push(
                    SptEmail2Img::parse(self)
                        .map(Tag::SptEmail2Img)
                        .map(Node::Tag)?,
                ),
                "spt_encryptemail_tag" => tags.push(
                    SptEncryptemail::parse(self)
                        .map(Tag::SptEncryptemail)
                        .map(Node::Tag)?,
                ),
                "spt_escapeemail_tag" => tags.push(
                    SptEscapeemail::parse(self)
                        .map(Tag::SptEscapeemail)
                        .map(Node::Tag)?,
                ),
                "spt_formsolutions_tag" => tags.push(
                    SptFormsolutions::parse(self)
                        .map(Tag::SptFormsolutions)
                        .map(Node::Tag)?,
                ),
                "spt_id2url_tag" => {
                    tags.push(SptId2Url::parse(self).map(Tag::SptId2Url).map(Node::Tag)?)
                }
                "spt_ilink_tag" => {
                    tags.push(SptIlink::parse(self).map(Tag::SptIlink).map(Node::Tag)?)
                }
                "spt_imageeditor_tag" => tags.push(
                    SptImageeditor::parse(self)
                        .map(Tag::SptImageeditor)
                        .map(Node::Tag)?,
                ),
                "spt_imp_tag" => tags.push(SptImp::parse(self).map(Tag::SptImp).map(Node::Tag)?),
                "spt_iterator_tag" => tags.push(
                    SptIterator::parse(self)
                        .map(Tag::SptIterator)
                        .map(Node::Tag)?,
                ),
                "spt_link_tag" => tags.push(SptLink::parse(self).map(Tag::SptLink).map(Node::Tag)?),
                "spt_number_tag" => {
                    tags.push(SptNumber::parse(self).map(Tag::SptNumber).map(Node::Tag)?)
                }
                "spt_personalization_tag" => tags.push(
                    SptPersonalization::parse(self)
                        .map(Tag::SptPersonalization)
                        .map(Node::Tag)?,
                ),
                "spt_prehtml_tag" => tags.push(
                    SptPrehtml::parse(self)
                        .map(Tag::SptPrehtml)
                        .map(Node::Tag)?,
                ),
                "spt_smarteditor_tag" => tags.push(
                    SptSmarteditor::parse(self)
                        .map(Tag::SptSmarteditor)
                        .map(Node::Tag)?,
                ),
                "spt_spml_tag" => tags.push(SptSpml::parse(self).map(Tag::SptSpml).map(Node::Tag)?),
                "spt_text_tag" => tags.push(SptText::parse(self).map(Tag::SptText).map(Node::Tag)?),
                "spt_textarea_tag" => tags.push(
                    SptTextarea::parse(self)
                        .map(Tag::SptTextarea)
                        .map(Node::Tag)?,
                ),
                "spt_timestamp_tag" => tags.push(
                    SptTimestamp::parse(self)
                        .map(Tag::SptTimestamp)
                        .map(Node::Tag)?,
                ),
                "spt_tinymce_tag" => tags.push(
                    SptTinymce::parse(self)
                        .map(Tag::SptTinymce)
                        .map(Node::Tag)?,
                ),
                "spt_updown_tag" => {
                    tags.push(SptUpdown::parse(self).map(Tag::SptUpdown).map(Node::Tag)?)
                }
                "spt_upload_tag" => {
                    tags.push(SptUpload::parse(self).map(Tag::SptUpload).map(Node::Tag)?)
                }
                "spt_worklist_tag" => tags.push(
                    SptWorklist::parse(self)
                        .map(Tag::SptWorklist)
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
        let node = self.cursor.node();
        let value;
        match node.kind() {
            "string_content" => {
                value = self.cursor.node().utf8_text(self.text_bytes)?.to_string();
                if !self.cursor.goto_next_sibling() {
                    return Err(anyhow::anyhow!("attribute value string is unclosed"));
                }
            }
            "\"" => value = "".to_string(),
            _ => return Err(anyhow::anyhow!("attribute value string is unclosed")),
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
        let node = self.cursor.node();
        let text;
        match node.kind() {
            "string_content" => {
                text = self.cursor.node().utf8_text(self.text_bytes)?.to_string();
                if !self.cursor.goto_next_sibling() {
                    return Err(anyhow::anyhow!("attribute value string is unclosed"));
                }
            }
            "\"" => text = "".to_string(),
            _ => return Err(anyhow::anyhow!("attribute value string is unclosed")),
        }
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
    return Location::new(
        start.column,
        start.row,
        node.end_position().column - start.column,
    );
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
            Header, Location, Node, PageHeader, PlainAttribute, SpBarcode, SpelAttribute, Tag,
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
            java_headers: vec![PageHeader {
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
            }],
            taglib_imports: vec![
                TagLibImport {
                    open_bracket: Location::new(2, 1, 3),
                    taglib: Location::new(6, 1, 6),
                    origin: TagLibOrigin::Uri(PlainAttribute {
                        key_location: Location::new(13, 1, 3),
                        equals_location: Location::new(16, 1, 1),
                        opening_quote_location: Location::new(17, 1, 1),
                        value: "http://www.sitepark.com/taglibs/core".to_string(),
                        closing_quote_location: Location::new(54, 1, 1),
                    }),
                    prefix: PlainAttribute {
                        key_location: Location::new(56, 1, 6),
                        equals_location: Location::new(62, 1, 1),
                        opening_quote_location: Location::new(63, 1, 1),
                        value: "sp".to_string(),
                        closing_quote_location: Location::new(66, 1, 1),
                    },
                    close_bracket: Location::new(0, 2, 2),
                },
                TagLibImport {
                    open_bracket: Location::new(2, 2, 3),
                    taglib: Location::new(6, 2, 6),
                    origin: TagLibOrigin::TagDir(PlainAttribute {
                        key_location: Location::new(13, 2, 6),
                        equals_location: Location::new(19, 2, 1),
                        opening_quote_location: Location::new(20, 2, 1),
                        value: "/WEB-INF/tags/spt".to_string(),
                        closing_quote_location: Location::new(38, 2, 1),
                    }),
                    prefix: PlainAttribute {
                        key_location: Location::new(40, 2, 6),
                        equals_location: Location::new(46, 2, 1),
                        opening_quote_location: Location::new(47, 2, 1),
                        value: "spt".to_string(),
                        closing_quote_location: Location::new(51, 2, 1),
                    },
                    close_bracket: Location::new(0, 3, 2),
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
        let expected = vec![Node::Tag(Tag::SpBarcode(SpBarcode {
            open_location: Location::new(0, 2, 11),
            height_attribute: None,
            locale_attribute: None,
            name_attribute: Some(SpelAttribute {
                key_location: Location::new(12, 2, 4),
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
            }),
            scope_attribute: Some(SpelAttribute {
                key_location: Location::new(46, 2, 5),
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
            }),
            text_attribute: Some(SpelAttribute {
                key_location: Location::new(29, 2, 4),
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
            }),
            type_attribute: None,
            body: None,
            close_location: Location::new(58, 2, 2),
        }))];
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
