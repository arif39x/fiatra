from typing import Any
from ..ir.math_expr import (
    Expr, Constant, Variable, Add, Sub, Mul, Div, Pow, Sin, Cos, Sqrt, Abs, Vec3
)

class ExprVisitor:
    """Base visitor for all Math IR expression nodes."""
    
    def visit(self, expr: Expr, *args, **kwargs) -> Any:
        method_name = f"visit_{type(expr).__name__}"
        visitor = getattr(self, method_name, self.generic_visit)
        return visitor(expr, *args, **kwargs)

    def generic_visit(self, expr: Expr, *args, **kwargs) -> Any:
        raise NotImplementedError(f"No visit_{type(expr).__name__} method defined in {type(self).__name__}")
        
class ExprTransformer(ExprVisitor):
    """Base visitor for transforming Math IR expression trees."""
    
    def generic_visit(self, expr: Expr, *args, **kwargs) -> Expr:
        return expr
        
    def visit_Constant(self, expr: Constant) -> Expr:
        return expr
        
    def visit_Variable(self, expr: Variable) -> Expr:
        return expr
        
    def visit_Add(self, expr: Add) -> Expr:
        return Add(left=self.visit(expr.left), right=self.visit(expr.right), span=expr.span, value_type=expr.value_type)
        
    def visit_Sub(self, expr: Sub) -> Expr:
        return Sub(left=self.visit(expr.left), right=self.visit(expr.right), span=expr.span, value_type=expr.value_type)
        
    def visit_Mul(self, expr: Mul) -> Expr:
        return Mul(left=self.visit(expr.left), right=self.visit(expr.right), span=expr.span, value_type=expr.value_type)
        
    def visit_Div(self, expr: Div) -> Expr:
        return Div(left=self.visit(expr.left), right=self.visit(expr.right), span=expr.span, value_type=expr.value_type)
        
    def visit_Pow(self, expr: Pow) -> Expr:
        return Pow(left=self.visit(expr.left), right=self.visit(expr.right), span=expr.span, value_type=expr.value_type)
        
    def visit_Sin(self, expr: Sin) -> Expr:
        return Sin(expr=self.visit(expr.expr), span=expr.span, value_type=expr.value_type)
        
    def visit_Cos(self, expr: Cos) -> Expr:
        return Cos(expr=self.visit(expr.expr), span=expr.span, value_type=expr.value_type)
        
    def visit_Sqrt(self, expr: Sqrt) -> Expr:
        return Sqrt(expr=self.visit(expr.expr), span=expr.span, value_type=expr.value_type)

    def visit_Abs(self, expr: Abs) -> Expr:
        return Abs(expr=self.visit(expr.expr), span=expr.span, value_type=expr.value_type)

    def visit_Vec3(self, expr: Vec3) -> Expr:
        return Vec3(x=self.visit(expr.x), y=self.visit(expr.y), z=self.visit(expr.z), span=expr.span, value_type=expr.value_type)
