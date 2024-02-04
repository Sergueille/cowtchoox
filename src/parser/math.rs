use crate::parser::custom::{CustomTag, TagHash};
use crate::parser::Node;


/// Create math! Called after tags are parsed. Will replace the provided Node's contents by math.
/// 
/// # Arguments
/// * `node`: The node, which content is raw, non-parsed math. It can contain HTML nodes however.
/// * `operators`: The "?" operators to use, indexed by their names
/// 
/// # Returns
/// The node that contains all the math.
/// 
pub fn parse_math(node: &mut Node, operators: TagHash) -> Result<(), ()> {
    todo!(); // TODO

    // NOTE: utiliser crate::parser::custom::instantiate_tag pour obtenir un node à partir de la struct de l'opérateur, et en donnant les arguments de l'opérateur
    // NOTE: Si il y a des sous-Nodes, appeler récursivement la fonction
    // NOTE: pour les erreurs, utiliser log::error sans spécifier de position, je ferai en sorte que ca l'indique plus tard

    // Et si il y a besoin, c'est possible de changer la spécification de la fonction 
}


