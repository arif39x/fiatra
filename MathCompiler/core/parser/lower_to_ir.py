import ast
from ..ir.math_expr import (
    Expr, Constant, Variable, Add, Sub, Mul, Div, Pow, Sin, Cos, Sqrt
)
from ..errors import SourceSpan, ParseError

def _get_span(node: ast.AST) -> SourceSpan | None:
    if hasattr(node, "lineno") and hasattr(node, "col_offset"):
        return SourceSpan(line=node.lineno, column=node.col_offset)
    return None

def lower_expr(node: ast.AST) -> Expr:
    """
    Lowers a Python AST node into the internal Math IR.
    """
    span = _get_span(node)
    
    if isinstance(node, ast.Expression):
        return lower_expr(node.body)
    
    if isinstance(node, ast.Constant):
        if not isinstance(node.value, (int, float)):
            raise ParseError(f"Unsupported constant type: {type(node.value).__name__}", span=span, node_type="Constant")
        return Constant(value=float(node.value), span=span)
    
    if isinstance(node, ast.Name):
        return Variable(name=node.id, span=span)
    
    if isinstance(node, ast.Attribute):
        def _get_attr_path(attr_node: ast.AST) -> str:
            if isinstance(attr_node, ast.Name):
                return attr_node.id
            if isinstance(attr_node, ast.Attribute):
                return f"{_get_attr_path(attr_node.value)}.{attr_node.attr}"
            raise ParseError(f"Unsupported attribute base: {type(attr_node).__name__}", span=_get_span(attr_node))
        
        path = f"{_get_attr_path(node.value)}.{node.attr}"
        return Variable(name=path, span=span)
    
    if isinstance(node, ast.BinOp):
        left = lower_expr(node.left)
        right = lower_expr(node.right)
        
        if isinstance(node.op, ast.Add):
            return Add(left=left, right=right, span=span)
        if isinstance(node.op, ast.Sub):
            return Sub(left=left, right=right, span=span)
        if isinstance(node.op, ast.Mult):
            return Mul(left=left, right=right, span=span)
        if isinstance(node.op, ast.Div):
            return Div(left=left, right=right, span=span)
        if isinstance(node.op, ast.Pow):
            return Pow(left=left, right=right, span=span)
        
        raise ParseError(f"Unsupported binary operator: {type(node.op).__name__}", span=span, node_type="BinOp")
    
    if isinstance(node, ast.UnaryOp):
        operand = lower_expr(node.operand)
        if isinstance(node.op, ast.USub):
            # Represent -x as -1 * x
            neg_one = Constant(value=-1.0, span=span)
            return Mul(left=neg_one, right=operand, span=span)
        if isinstance(node.op, ast.UAdd):
            return operand
        raise ParseError(f"Unsupported unary operator: {type(node.op).__name__}", span=span, node_type="UnaryOp")

    if isinstance(node, ast.Call):
        if not isinstance(node.func, ast.Name):
            raise ParseError("Unsupported function call type", span=span, node_type="Call")
        
        func_name = node.func.id.lower()
        if len(node.args) != 1:
            raise ParseError(f"Function {func_name} expects exactly 1 argument", span=span, node_type="Call")
        
        arg = lower_expr(node.args[0])
        
        if func_name == "sin":
            return Sin(expr=arg, span=span)
        if func_name == "cos":
            return Cos(expr=arg, span=span)
        if func_name == "sqrt":
            return Sqrt(expr=arg, span=span)
        
        if func_name in ("max", "min"):
            if len(node.args) != 2:
                raise ParseError(f"Function {func_name} expects exactly 2 arguments", span=span, node_type="Call")
            left = lower_expr(node.args[0])
            right = lower_expr(node.args[1])
            if func_name == "max":
                return Max(left=left, right=right, span=span)
            return Min(left=left, right=right, span=span)
        
        raise ParseError(f"Unsupported function: {func_name}", span=span, node_type="Call")

    raise ParseError(f"Unsupported AST node: {type(node).__name__}", span=span, node_type=type(node).__name__)

def parse_string_to_ir(expr_str: str) -> Expr:
    """Helper to parse a string directly to Math IR."""
    try:
        tree = ast.parse(expr_str, mode='eval')
        return lower_expr(tree)
    except SyntaxError as e:
        raise ParseError(f"Syntax error: {e.msg}", span=SourceSpan(line=e.lineno or 1, column=e.offset or 1))
