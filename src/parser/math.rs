use super::{Node, ParserContext, ParseError};

/*

How it works:
`parser::parse_tag` <- takes raw text
    - parse comments, </>, backslash etc...
    - make a recursive call for child tags
    - call `parse_math` just before exiting

Example:

<math>§D = b^2-4ac = <boxed>§w_0^2(1/{Q^2}-4)</boxed></math>

- parser::parse_tag called here: 
    <math>§D = b^2-4ac = <div>§w_0^2(1/{Q^2}-4)</div></math>
    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    - recursive call to parser::parse_tag
        <math>§D = b^2-4ac = <div>§w_0^2(1/{Q^2}-4)</div></math>
                             ~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    - call to parse_math here then exit
        <math>§D = b^2-4ac = <div>§w_0^2(1/{Q^2}-4)</div></math>
                             ~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- call to parse_math here then exit
    <math>§D = b^2-4ac = <div>[HTML representation of the math]</div></math>
    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*/


/// Create math! Called after tags are parsed. Will replace the provided Node's contents by math.
/// 
/// # Arguments
/// * `node`: A node. It's children are fully math-parsed, but not it's inner text
/// 
/// # Returns
/// The node that contains all the math.
/// 
pub fn parse_math(node: &mut Node, context: &ParserContext) -> Result<(), ParseError> {
    // TODO

    // NOTE: les opérateurs définis par l'utilisateur sont dans `context`, dans un dictionnaire, indexés par leur nom 
    //       utiliser crate::parser::custom::instantiate_tag pour obtenir un node à partir de la struct de l'opérateur, et en donnant les arguments de l'opérateur
    // NOTE: pour les erreurs, utiliser log::error sans spécifier de position, je ferai en sorte que ca l'indique plus tard

    // Et si il y a besoin, c'est possible de changer la spécification de la fonction 

    return Ok(());
}


