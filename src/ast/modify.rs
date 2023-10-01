use std::collections::HashMap;

use super::{
    expressions::{
        ArrayLiteral, Boolean, CallExpression, FunctionLiteral, HashLiteral, Identifier,
        IfExpression, IndexExpression, InfixExpression, IntegerLiteral, MacroLiteral,
        PrefixExpression, StringLiteral,
    },
    program::Program,
    statements::{BlockStatement, ExpressionStatement, LetStatement, ReturnStatement},
    traits::{AsNode, Expression, Node, Statement},
};

// rust 里面没有办法把一个 &dyn Trait 转换为另一个，因为 &dyn Trait 里面存放的 vtable 没办法替换
// 我开始想的是直接替换，就都没有返回值，但调用 modify 的时候没有办法在 modifier 这个参数里面用一个新的 node 替换旧的参数 node
pub fn modify(
    node: &mut dyn Node,
    modifier: &impl Fn(Box<dyn Node>) -> Box<dyn Node>,
) -> Box<dyn Node> {
    if let Some(program) = node.downcast_mut::<Program>() {
        for statement in program.statements.iter_mut() {
            *statement = node_to_statement_helper(modify(statement.as_mut_node(), modifier));
        }
    } else if let Some(expression_statement) = node.downcast_mut::<ExpressionStatement>() {
        expression_statement.expression = node_to_expression_helper(modify(
            expression_statement.expression.as_mut_node(),
            modifier,
        ));
    } else if let Some(block_statement) = node.downcast_mut::<BlockStatement>() {
        for statement in block_statement.statements.iter_mut() {
            *statement = node_to_statement_helper(modify(statement.as_mut_node(), modifier));
        }
    } else if let Some(return_statement) = node.downcast_mut::<ReturnStatement>() {
        return_statement.return_value = node_to_expression_helper(modify(
            return_statement.return_value.as_mut_node(),
            modifier,
        ));
    } else if let Some(let_statement) = node.downcast_mut::<LetStatement>() {
        let_statement.value =
            node_to_expression_helper(modify(let_statement.value.as_mut_node(), modifier));
    } else if let Some(infix_expresssion) = node.downcast_mut::<InfixExpression>() {
        infix_expresssion.left =
            node_to_expression_helper(modify(infix_expresssion.left.as_mut_node(), modifier));
        infix_expresssion.right =
            node_to_expression_helper(modify(infix_expresssion.right.as_mut_node(), modifier));
    } else if let Some(prefix_expression) = node.downcast_mut::<PrefixExpression>() {
        prefix_expression.right =
            node_to_expression_helper(modify(prefix_expression.right.as_mut_node(), modifier));
    } else if let Some(index_expresssion) = node.downcast_mut::<IndexExpression>() {
        index_expresssion.left =
            node_to_expression_helper(modify(index_expresssion.left.as_mut_node(), modifier));
        index_expresssion.index =
            node_to_expression_helper(modify(index_expresssion.index.as_mut_node(), modifier));
    } else if let Some(if_expression) = node.downcast_mut::<IfExpression>() {
        if_expression.condition =
            node_to_expression_helper(modify(if_expression.condition.as_mut_node(), modifier));
        modify(if_expression.consequence.as_mut_node(), modifier)
            .downcast::<BlockStatement>()
            .map_err(|_| "Shouldn't happen")
            .unwrap();
        if let Some(alternative) = if_expression.alternative.as_mut() {
            if_expression.alternative = Some(
                *modify(alternative.as_mut_node(), modifier)
                    .downcast::<BlockStatement>()
                    .map_err(|_| "Shouldn't happen")
                    .unwrap(),
            );
        }
    } else if let Some(function_literal) = node.downcast_mut::<FunctionLiteral>() {
        for ident in function_literal.parameters.iter_mut() {
            *ident = *modify(ident.as_mut_node(), modifier)
                .downcast::<Identifier>()
                .map_err(|_| "Shouldn't happen")
                .unwrap();
        }
        function_literal.body = *modify(function_literal.body.as_mut_node(), modifier)
            .downcast::<BlockStatement>()
            .map_err(|_| "Shouldn't happen")
            .unwrap();
    } else if let Some(array_literal) = node.downcast_mut::<ArrayLiteral>() {
        for element in array_literal.elements.iter_mut() {
            modify(element.as_mut_node(), modifier);
        }
    } else if let Some(hash_literal) = node.downcast_mut::<HashLiteral>() {
        let mut new_pairs = HashMap::new();
        for (key, value) in hash_literal.pairs.iter() {
            let mut new_key = dyn_clone::clone(&**key);
            modify(new_key.as_mut_node(), modifier);
            let mut new_value = dyn_clone::clone_box(value.as_ref());
            modify(new_value.as_mut_node(), modifier);
            new_pairs.insert(by_address::ByAddress(new_key), new_value);
        }
        hash_literal.pairs = new_pairs;
    }
    let node = dyn_clone::clone_box(node);
    modifier(node)
}

fn node_to_statement_helper(node: Box<dyn Node>) -> Box<dyn Statement> {
    if let Some(let_statement) = node.downcast_ref::<LetStatement>() {
        dyn_clone::clone_box(let_statement)
    } else if let Some(return_statement) = node.downcast_ref::<ReturnStatement>() {
        dyn_clone::clone_box(return_statement)
    } else if let Some(expression_statement) = node.downcast_ref::<ExpressionStatement>() {
        dyn_clone::clone_box(expression_statement)
    } else {
        dyn_clone::clone_box(node.downcast_ref::<BlockStatement>().unwrap())
    }
}

fn node_to_expression_helper(node: Box<dyn Node>) -> Box<dyn Expression> {
    if let Some(ident) = node.downcast_ref::<Identifier>() {
        dyn_clone::clone_box(ident)
    } else if let Some(integer) = node.downcast_ref::<IntegerLiteral>() {
        dyn_clone::clone_box(integer)
    } else if let Some(boolean) = node.downcast_ref::<Boolean>() {
        dyn_clone::clone_box(boolean)
    } else if let Some(if_exp) = node.downcast_ref::<IfExpression>() {
        dyn_clone::clone_box(if_exp)
    } else if let Some(func) = node.downcast_ref::<FunctionLiteral>() {
        dyn_clone::clone_box(func)
    } else if let Some(call) = node.downcast_ref::<CallExpression>() {
        dyn_clone::clone_box(call)
    } else if let Some(infix) = node.downcast_ref::<InfixExpression>() {
        dyn_clone::clone_box(infix)
    } else if let Some(prefix) = node.downcast_ref::<PrefixExpression>() {
        dyn_clone::clone_box(prefix)
    } else if let Some(str) = node.downcast_ref::<StringLiteral>() {
        dyn_clone::clone_box(str)
    } else if let Some(arr) = node.downcast_ref::<ArrayLiteral>() {
        dyn_clone::clone_box(arr)
    } else if let Some(hash) = node.downcast_ref::<HashLiteral>() {
        dyn_clone::clone_box(hash)
    } else if let Some(macro_literal) = node.downcast_ref::<MacroLiteral>() {
        dyn_clone::clone_box(macro_literal)
    } else {
        dyn_clone::clone_box(node.downcast_ref::<IndexExpression>().unwrap())
    }
}
