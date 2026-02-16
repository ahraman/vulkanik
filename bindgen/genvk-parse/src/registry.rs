use std::{collections::HashMap, io::Read};

use genvk_macros::{FromAttr, Xml};
use xml::{EventReader, reader::XmlEvent};

use crate::{
    Error,
    traits::{FromAttr, IntoMap, MapExt, PushText},
};

#[derive(Debug, FromAttr)]
pub enum Api {
    Vulkan,
    VulkanSc,
    VulkanBase,
}

#[derive(Debug, FromAttr)]
pub enum Deprecation {
    False,
    True,
    Ignored,
    Aliased,
}

#[derive(Debug)]
pub struct Depends(pub String);

impl FromAttr for Depends {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        match value {
            None => Err(None),
            Some(value) => Ok(Self(value)),
        }
    }
}

#[derive(Debug, Xml)]
#[xml(text)]
pub struct Comment(#[xml(text)] pub String);

#[derive(Debug)]
pub enum Content {
    Characters(String),
    Name(String),
    Type(String),
    Enum(String),
    Comment(String),
}

impl Content {
    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        attributes.check_empty(&element, reader)?;

        let mut content = String::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    return if name.local_name == element {
                        Ok(match element.as_str() {
                            "name" => Self::Name(content),
                            "type" => Self::Type(content),
                            "enum" => Self::Enum(content),
                            "comment" => Self::Comment(content),
                            _ => return Err(Error::UnknownStart(element)),
                        })
                    } else {
                        Err(Error::UnexpectedEnd(name.local_name, element))
                    };
                }
                XmlEvent::Characters(text) => content += text.as_str(),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }
    }
}

impl PushText for Vec<Content> {
    fn push_text(&mut self, text: String) {
        self.push(Content::Characters(text));
    }
}

#[derive(Debug, Xml)]
#[xml(items(RegistryItem))]
pub struct Registry {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<RegistryItem>,
}

#[derive(Debug, Xml)]
pub enum RegistryItem {
    Comment(Comment),
    Platforms(Platforms),
    Tags(Tags),
    Types(Types),
    Enums(Enums),
    Commands(Commands),
    Formats(Formats),
    SpirvExtensions(SpirvExtensions),
    SpirvCapabilities(SpirvCapabilities),
    Sync(Sync),
    VideoCodecs(VideoCodecs),
}

#[derive(Debug, Xml)]
#[xml(items(Platform))]
pub struct Platforms {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<Platform>,
}

#[derive(Debug, Xml)]
pub struct Platform {
    pub name: String,
    pub protect: String,
    pub comment: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(items(Tag))]
pub struct Tags {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<Tag>,
}

#[derive(Debug, Xml)]
pub struct Tag {
    pub name: String,
    pub author: String,
    pub contact: String,
    pub comment: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(items(TypesItem))]
pub struct Types {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<TypesItem>,
}

#[derive(Debug, Xml)]
pub enum TypesItem {
    Comment(Comment),
    Type(Type),
}

#[derive(Debug, Xml)]
#[xml(inline)]
pub struct Type {
    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub comment: Option<String>,

    #[xml(inner)]
    pub kind: TypeKind,
}

#[derive(Debug, Xml)]
#[xml(attr("category"), incomplete)]
pub enum TypeKind {
    #[xml(none)]
    External(ExternalType),
    #[xml(value = "include")]
    Include(IncludeType),
    #[xml(value = "define")]
    Define(DefineType),
    #[xml(value = "basetype")]
    Base(BaseType),
    #[xml(value = "bitmask")]
    Bitmask(BitmaskType),
    #[xml(value = "handle")]
    Handle(HandleType),
    #[xml(value = "enum")]
    Enum(EnumType),
    #[xml(value = "funcpointer")]
    FnPtr(FnPtrType),
    #[xml(value = "struct")]
    Struct(StructType),
    #[xml(value = "union")]
    Union(StructType),
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct TypeAlias {
    pub name: String,
    pub alias: String,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct ExternalType {
    pub name: String,
}

#[derive(Debug, Xml)]
#[xml(text, incomplete)]
pub struct IncludeType {
    pub name: String,

    #[xml(text)]
    pub content: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(mixed(Content), incomplete)]
pub struct DefineType {
    pub name: Option<String>,

    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(mixed(Content), incomplete)]
pub struct BaseType {
    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(attr, incomplete)]
pub enum BitmaskType {
    #[xml(default)]
    Decl(BitmaskTypeDecl),
    #[xml(attr = "alias")]
    Alias(TypeAlias),
}

#[derive(Debug, Xml)]
#[xml(mixed(Content), incomplete)]
pub struct BitmaskTypeDecl {
    #[xml(rename = "bitvalues")]
    pub bit_values: Option<String>,

    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(attr, incomplete)]
pub enum HandleType {
    #[xml(default)]
    Decl(HandleTypeDecl),
    #[xml(attr = "alias")]
    Alias(TypeAlias),
}

#[derive(Debug, Xml)]
#[xml(mixed(Content), incomplete)]
pub struct HandleTypeDecl {
    pub parent: Option<String>,
    #[xml(rename = "objtypeenum")]
    pub obj_type_enum: String,

    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(attr, incomplete)]
pub enum EnumType {
    #[xml(default)]
    Decl(EnumTypeDecl),
    #[xml(attr = "alias")]
    Alias(TypeAlias),
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct EnumTypeDecl {
    pub name: String,
}

#[derive(Debug, Xml)]
#[xml(items(CommandItem), incomplete)]
pub struct FnPtrType {
    #[xml(items)]
    pub items: Vec<CommandItem>,
}

#[derive(Debug, Xml)]
#[xml(attr, incomplete)]
pub enum StructType {
    #[xml(default)]
    Decl(StructTypeDecl),
    #[xml(attr = "alias")]
    Alias(TypeAlias),
}

#[derive(Debug, Xml)]
#[xml(items(StructItem), incomplete)]
pub struct StructTypeDecl {
    pub name: String,

    #[xml(rename = "allowduplicate")]
    pub allow_duplicate: Option<bool>,
    #[xml(rename = "requiredlimittype")]
    pub required_limit_type: Option<bool>,
    #[xml(rename = "returnedonly")]
    pub returned_only: Option<bool>,
    #[xml(rename = "structextends")]
    pub struct_extends: Option<Vec<String>>,

    #[xml(items)]
    pub items: Vec<StructItem>,
}

#[derive(Debug, Xml)]
pub enum StructItem {
    Comment(Comment),
    Member(Member),
}

#[derive(Debug)]
pub enum Len {
    NullTerminated,
    One,
    Member(String),
    LatexMath(String),
}

impl FromAttr for Len {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        match value {
            None => Err(None),
            Some(value) => Ok(match value.as_str() {
                "null-terminated" => Self::NullTerminated,
                "1" => Self::One,
                _ => {
                    if value.starts_with("latexmath:") {
                        Self::LatexMath(value)
                    } else {
                        Self::Member(value)
                    }
                }
            }),
        }
    }
}

#[derive(Debug, FromAttr)]
pub enum ExternSync {
    False,
    True,
    Maybe,
}

#[derive(Debug, FromAttr)]
pub enum LimitType {
    Min,
    Max,
    #[attr(rename = "pot")]
    PowerOfTwo,
    Mul,
    Bits,
    Bitmask,
    Range,
    Struct,
    Exact,
    NoAuto,
}

#[derive(Debug, Xml)]
#[xml(mixed(Content))]
pub struct Member {
    pub api: Option<Vec<Api>>,
    pub optional: Option<Vec<bool>>,
    pub deprecated: Option<Deprecation>,

    pub values: Option<Vec<String>>,
    #[xml(rename = "externsync")]
    pub extern_sync: Option<ExternSync>,
    #[xml(rename = "noautovalidity")]
    pub no_auto_validity: Option<bool>,

    pub len: Option<Vec<Len>>,
    #[xml(rename = "altlen")]
    pub alt_len: Option<String>,
    #[xml(rename = "objecttype")]
    pub object_type: Option<String>,

    #[xml(rename = "limittype")]
    pub limit_type: Option<Vec<LimitType>>,
    #[xml(rename = "featurelink")]
    pub feature_link: Option<String>,

    pub selector: Option<String>,
    pub selection: Option<Vec<String>>,

    #[xml(content)]
    pub contents: Vec<Content>,
}

#[derive(Debug, FromAttr)]
pub enum EnumsType {
    Constants,
    Bitmask,
    Enum,
}

#[derive(Debug, Xml)]
#[xml(items(EnumsItem))]
pub struct Enums {
    pub name: Option<String>,
    #[xml(rename = "type")]
    pub r#type: EnumsType,
    #[xml(rename = "bitwidth")]
    pub bit_width: Option<i32>,
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<EnumsItem>,
}

#[derive(Debug, Xml)]
pub enum EnumsItem {
    Comment(Comment),
    Enum(Enum),
    Unused(Unused),
}

#[derive(Debug, Xml)]
#[xml(inline)]
pub struct Enum {
    pub api: Option<Vec<Api>>,
    pub deprecated: Option<Deprecation>,
    pub protect: Option<String>,
    pub comment: Option<String>,

    #[xml(inner)]
    pub kind: EnumKind,
}

#[derive(Debug, Xml)]
#[xml(attr, incomplete)]
pub enum EnumKind {
    #[xml(attr = "value")]
    Value(EnumValue),
    #[xml(attr = "bitpos")]
    BitPos(EnumBitPos),
    #[xml(attr = "alias")]
    Alias(EnumAlias),
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct EnumValue {
    pub name: String,
    pub value: String,
    #[xml(rename = "type")]
    pub r#type: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct EnumBitPos {
    pub name: String,
    #[xml(rename = "bitpos")]
    pub bit_pos: String,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct EnumAlias {
    pub name: String,
    pub alias: String,
}

#[derive(Debug, Xml)]
pub struct Unused {
    pub start: String,
    pub end: Option<String>,
    pub vendor: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(items(CommandsItem))]
pub struct Commands {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<CommandsItem>,
}

#[derive(Debug, Xml)]
pub enum CommandsItem {
    Comment(Comment),
    Command(Command),
}

#[derive(Debug, Xml)]
#[xml(inline)]
pub struct Command {
    pub api: Option<Vec<Api>>,
    pub comment: Option<String>,

    #[xml(inner)]
    pub kind: CommandKind,
}

#[derive(Debug, Xml)]
#[xml(attr, incomplete)]
pub enum CommandKind {
    #[xml(default)]
    Decl(CommandDecl),
    #[xml(attr = "alias")]
    Alias(CommandAlias),
}

#[derive(Debug, Xml)]
#[xml(items(CommandItem), incomplete)]
pub struct CommandDecl {
    #[xml(items)]
    pub items: Vec<CommandItem>,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct CommandAlias {
    pub name: String,
    pub alias: String,
}

#[derive(Debug, Xml)]
pub enum CommandItem {
    Proto(Proto),
    Param(Param),
}

#[derive(Debug, Xml)]
#[xml(mixed(Content))]
pub struct Proto {
    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(mixed(Content))]
pub struct Param {
    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(items(Format))]
pub struct Formats {
    #[xml(items)]
    pub items: Vec<Format>,
}

#[derive(Debug, Xml)]
#[xml(items(FormatItem))]
pub struct Format {
    pub name: String,
    pub class: String,
    #[xml(rename = "blockSize")]
    pub block_size: String,
    #[xml(rename = "texelsPerBlock")]
    pub texels_per_block: String,
    #[xml(rename = "blockExtent")]
    pub block_extent: Option<Vec<i32>>,
    pub packed: Option<i32>,
    pub compressed: Option<String>,
    pub chroma: Option<String>,

    #[xml(items)]
    pub items: Vec<FormatItem>,
}

#[derive(Debug, Xml)]
pub enum FormatItem {
    Component(Component),
    Plane(Plane),
    SpirvImageFormat(SpirvImageFormat),
}

#[derive(Debug)]
pub enum Bits {
    Compressed,
    Value(i32),
}

impl FromAttr for Bits {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        Ok(match value.as_ref().map(|value| value.as_str()) {
            Some("compressed") => Self::Compressed,
            _ => Self::Value(i32::from_attr(value)?),
        })
    }
}

#[derive(Debug, Xml)]
pub struct Component {
    pub name: String,
    pub bits: Bits,
    #[xml(rename = "numericFormat")]
    pub numeric_format: String,
    #[xml(rename = "planeIndex")]
    pub plane_index: Option<i32>,
}

#[derive(Debug, Xml)]
pub struct Plane {
    pub index: i32,
    #[xml(rename = "widthDivisor")]
    pub width_divisor: Bits,
    #[xml(rename = "heightDivisor")]
    pub height_divisor: Bits,
    pub compatible: String,
}

#[derive(Debug, Xml)]
pub struct SpirvImageFormat {
    pub name: String,
}

#[derive(Debug, Xml)]
#[xml(items(SpirvExtension))]
pub struct SpirvExtensions {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<SpirvExtension>,
}

#[derive(Debug, Xml)]
#[xml(items(SpirvEnable))]
pub struct SpirvExtension {
    pub name: String,

    #[xml(items)]
    pub enables: Vec<SpirvEnable>,
}

#[derive(Debug, Xml)]
#[xml(items(SpirvCapability))]
pub struct SpirvCapabilities {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<SpirvCapability>,
}

#[derive(Debug, Xml)]
#[xml(items(SpirvEnable))]
pub struct SpirvCapability {
    pub name: String,

    #[xml(items)]
    pub enables: Vec<SpirvEnable>,
}

#[derive(Debug, Xml)]
#[xml(attr, rename = "enable")]
pub enum SpirvEnable {
    #[xml(attr = "version")]
    Version(SpirvEnableVersion),
    #[xml(attr = "extension")]
    Extension(SpirvEnableExtension),
    #[xml(attr = "feature")]
    Feature(SpirvEnableFeature),
    #[xml(attr = "property")]
    Property(SpirvEnableProperty),
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct SpirvEnableVersion {
    pub version: String,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct SpirvEnableExtension {
    pub extension: String,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct SpirvEnableFeature {
    #[xml(rename = "struct")]
    pub r#struct: String,
    pub feature: String,
    pub requires: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct SpirvEnableProperty {
    pub property: String,
    pub member: String,
    pub value: String,
    pub requires: Option<Vec<String>>,
}

#[derive(Debug, Xml)]
#[xml(items(SyncItem))]
pub struct Sync {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<SyncItem>,
}

#[derive(Debug, Xml)]
pub enum SyncItem {
    Comment(Comment),
    SyncStage(SyncStage),
    SyncAccess(SyncAccess),
    SyncPipeline(SyncPipeline),
}

#[derive(Debug, Xml)]
#[xml(items(SyncStageItem))]
pub struct SyncStage {
    pub name: String,
    pub alias: Option<String>,

    #[xml(items)]
    pub items: Vec<SyncStageItem>,
}

#[derive(Debug, Xml)]
pub enum SyncStageItem {
    Comment(Comment),
    SyncSupport(SyncSupportStage),
    SyncEquivalent(SyncEquivalentStage),
}

#[derive(Debug, Xml)]
#[xml(rename = "syncsupport")]
pub struct SyncSupportStage {
    pub queues: Vec<String>,
}

#[derive(Debug, Xml)]
#[xml(rename = "syncequivalent")]
pub struct SyncEquivalentStage {
    pub stage: Vec<String>,
}

#[derive(Debug, Xml)]
#[xml(items(SyncAccessItem))]
pub struct SyncAccess {
    pub name: String,
    pub alias: Option<String>,

    #[xml(items)]
    pub items: Vec<SyncAccessItem>,
}

#[derive(Debug, Xml)]
pub enum SyncAccessItem {
    Comment(Comment),
    SyncSupport(SyncSupportAccess),
    SyncEquivalent(SyncEquivalentAccess),
}

#[derive(Debug, Xml)]
#[xml(rename = "syncsupport")]
pub struct SyncSupportAccess {
    pub stage: Vec<String>,
}

#[derive(Debug, Xml)]
#[xml(rename = "syncequivalent")]
pub struct SyncEquivalentAccess {
    pub access: Vec<String>,
}

#[derive(Debug, Xml)]
#[xml(items(SyncPipelineStage))]
pub struct SyncPipeline {
    pub name: String,
    pub depends: Option<Depends>,

    #[xml(items)]
    pub stages: Vec<SyncPipelineStage>,
}

#[derive(Debug, Xml)]
#[xml(text)]
pub struct SyncPipelineStage {
    pub order: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,

    #[xml(text)]
    pub name: String,
}

#[derive(Debug, Xml)]
#[xml(items(VideoCodec))]
pub struct VideoCodecs {
    #[xml(items)]
    pub items: Vec<VideoCodec>,
}

#[derive(Debug, Xml)]
#[xml(items(VideoCodecItem))]
pub struct VideoCodec {
    pub name: String,
    pub extend: Option<String>,
    pub value: Option<String>,

    #[xml(items)]
    pub items: Vec<VideoCodecItem>,
}

#[derive(Debug, Xml)]
pub enum VideoCodecItem {
    VideoProfiles(VideoProfiles),
    VideoCapabilities(VideoCapabilities),
    VideoFormat(VideoFormat),
}

#[derive(Debug, Xml)]
#[xml(items(VideoProfileMember))]
pub struct VideoProfiles {
    #[xml(rename = "struct")]
    pub r#struct: String,

    #[xml(items)]
    pub members: Vec<VideoProfileMember>,
}

#[derive(Debug, Xml)]
#[xml(items(VideoProfile))]
pub struct VideoProfileMember {
    pub name: String,

    #[xml(items)]
    pub profiles: Vec<VideoProfile>,
}

#[derive(Debug, Xml)]
pub struct VideoProfile {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Xml)]
pub struct VideoCapabilities {
    #[xml(rename = "struct")]
    pub r#struct: String,
}

#[derive(Debug, Xml)]
#[xml(items(VideoFormatItem))]
pub struct VideoFormat {
    pub name: Option<String>,
    pub extend: Option<String>,
    pub usage: Option<String>, // more complex parsing required, see registry reference

    #[xml(items)]
    pub items: Vec<VideoFormatItem>,
}

#[derive(Debug, Xml)]
pub enum VideoFormatItem {
    VideoRequireCapabilities(VideoRequireCapabilities),
    VideoFormatProperties(VideoFormatProperties),
}

#[derive(Debug, Xml)]
pub struct VideoRequireCapabilities {
    #[xml(rename = "struct")]
    pub r#struct: String,
    pub member: String,
    pub value: String, // more complex parsing required, see registry reference
}

#[derive(Debug, Xml)]
pub struct VideoFormatProperties {
    #[xml(rename = "struct")]
    pub r#struct: String,
}
