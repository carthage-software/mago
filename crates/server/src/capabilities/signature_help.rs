//! Signatures and the active parameter for the call around the cursor.

use mago_bytes::BytesDisplay;
use mago_codex::ttype::display::NameDisplay;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_word::Word;

use crate::Server;
use crate::call_analysis::cursor::CallCursor;
use crate::call_analysis::cursor::enclosing_call;
use crate::call_analysis::signature::CallSignature;
use crate::call_analysis::targets::resolve_signature_targets;
use crate::documentation::Documentation;
use crate::domain::SignatureInfo;
use crate::lookup;

impl Server {
    /// The first known signature of the call around `offset`.
    pub fn get_signature_help(&mut self, file_id: FileId, offset: u32) -> Option<SignatureInfo> {
        self.get_signatures(file_id, offset).into_iter().next()
    }

    /// All known signatures, including distinct methods on a union receiver.
    pub fn get_signatures(&mut self, file_id: FileId, offset: u32) -> Vec<SignatureInfo> {
        let Ok(file) = self.database().get(&file_id) else { return Vec::new() };
        let tokens: Vec<_> = self.lex(&file).into_iter().filter(|token| !lookup::is_trivia(token.kind)).collect();
        let Some(call) = enclosing_call(&tokens, offset) else { return Vec::new() };
        let targets = resolve_signature_targets(self, file_id, &tokens, &call);
        let names = super::signature::call_type_names(self.codebase(), &targets);
        let mut documentation = Documentation::new(self);
        let mut signatures: Vec<_> = targets
            .into_iter()
            .filter_map(|target| {
                target.signature(self.codebase()).map(|signature| {
                    let metadata = signature.function;
                    let mut info = signature_info(
                        self.database(),
                        signature,
                        &names,
                        target.owner.as_ref().map(Word::as_bytes),
                        target.name.unwrap_or(metadata.original_name).as_bytes(),
                        &call,
                    );
                    info.documentation = documentation.for_callable(target.key);
                    info
                })
            })
            .collect();
        signatures.sort_unstable_by(|left, right| left.label.cmp(&right.label));
        for index in (0..signatures.len()).rev() {
            let signature = &signatures[index];
            if signatures[..index]
                .iter()
                .rev()
                .take_while(|previous| previous.label == signature.label)
                .any(|previous| previous.documentation == signature.documentation)
            {
                signatures.remove(index);
            }
        }
        signatures
    }
}

fn signature_info(
    database: &Database<'_>,
    signature: CallSignature<'_>,
    names: &NameDisplay<'_>,
    class: Option<&[u8]>,
    name: &[u8],
    call: &CallCursor<'_>,
) -> SignatureInfo {
    use std::fmt::Write;
    let meta = signature.function;
    let mut label = String::from("function ");
    if meta.flags.is_by_reference() {
        label.push('&');
    }
    if let Some(class) = class {
        let _ = write!(label, "{}::", BytesDisplay(class));
    }
    if !meta.kind.is_closure() && !meta.kind.is_arrow_function() {
        let _ = write!(label, "{}", BytesDisplay(name));
    }
    label.push('(');
    let mut parameters = Vec::with_capacity(signature.len());
    for (index, param) in signature.parameters().enumerate() {
        if index > 0 {
            label.push_str(", ");
        }
        let displayed_type = param.type_name(Some(names));
        parameters.push(super::signature::append_parameter_with_flags(
            &mut label,
            database,
            param.declaration,
            displayed_type.as_deref(),
            param.flags(),
        ));
    }
    label.push(')');
    let return_type = signature.return_type(Some(names));
    if let Some(ty) = return_type {
        let _ = write!(label, ": {}", BytesDisplay(ty.as_ref()));
    }
    let active_parameter = call
        .argument_name
        .and_then(|name| {
            signature
                .parameters()
                .position(|parameter| parameter.declaration.name.0.as_bytes().strip_prefix(b"$") == Some(name))
                .or_else(|| signature.parameters().position(|parameter| parameter.flags().is_variadic()))
        })
        .map_or(call.argument, |index| index as u32)
        .min(signature.len().saturating_sub(1) as u32);
    SignatureInfo { label, parameters, active_parameter, documentation: None }
}
