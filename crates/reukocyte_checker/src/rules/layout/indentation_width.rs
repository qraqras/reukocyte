use crate::checker::Checker;
use crate::diagnostic::{Edit, Fix, Severity};
use crate::rule::{LayoutRule, RuleId};
use ruby_prism::Node;

/// Rule identifier for Layout/IndentationWidth.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::IndentationWidth);

/// Default indentation width (2 spaces).
pub const DEFAULT_WIDTH: i32 = 2;

/// Check assignment nodes for indentation width violations.
pub fn check_assignment(node: &Node, rhs: &Node, checker: &mut Checker) {
    // def check_assignment(node, rhs)
    //   # If there are method calls chained to the right hand side of the
    //   # assignment, we let rhs be the receiver of those method calls before
    //   # we check its indentation.
    //   rhs = first_part_of_call_chain(rhs)
    //   return unless rhs

    //   end_config = config.for_cop('Layout/EndAlignment')
    //   style = end_config['EnforcedStyleAlignWith'] || 'keyword'
    //   base = variable_alignment?(node.loc, rhs, style.to_sym) ? node : rhs

    //   case rhs.type
    //   when :if            then on_if(rhs, base)
    //   when :while, :until then on_while(rhs, base)
    //   else                     return
    //   end

    //   ignore_node(rhs)
    // end
}
