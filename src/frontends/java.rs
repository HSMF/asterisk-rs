use super::Visitor;

pub struct Java {
    prelude: String,
    class_name: String
}

impl Visitor for Java {
    fn before_enter(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter, all_states: &[crate::generator::Uid]) -> std::fmt::Result {
        writeln!(f, "public class {} {{", self.class_name)?;
        writeln!(f, "private static enum State {{")?;
        for state in all_states {
            writeln!(f, "STATE_{state},")?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }

    fn after_leave(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter, all_states: &[crate::generator::Uid]) -> std::fmt::Result {
        writeln!(f, "}}")?; // public class
        Ok(())
    }

    fn begin_parse_loop(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "while (true) {{")?;
        Ok(())
    }

    fn end_parse_loop(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "}}")?; // while true
        Ok(())
    }

    fn enter_state(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter, state: crate::generator::Uid) -> std::fmt::Result {
        todo!()
    }

    fn leave_state(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter, state: crate::generator::Uid) -> std::fmt::Result {
        todo!()
    }

    fn enter_match(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter, state: crate::generator::Uid, token: crate::grammar::Token) -> std::fmt::Result {
        todo!()
    }

    fn leave_match(&self, ctx: &super::Ctx, f: &mut std::fmt::Formatter, state: crate::generator::Uid, token: crate::grammar::Token) -> std::fmt::Result {
        todo!()
    }

    fn visit_shift(
        &self,
        ctx: &super::Ctx,
        f: &mut std::fmt::Formatter,
        state: crate::generator::Uid,
        token: crate::grammar::Token,
        next_state: crate::generator::Uid,
    ) -> std::fmt::Result {
        todo!()
    }

    fn visit_reduce(
        &self,
        ctx: &super::Ctx,
        f: &mut std::fmt::Formatter,
        state: crate::generator::Uid,
        token: crate::grammar::Token,
        rule: crate::string_pool::Id,
        expansion: &[crate::grammar::Token],
    ) -> std::fmt::Result {
        todo!()
    }

    fn matching_error(
        &self,
        ctx: &super::Ctx,
        f: &mut std::fmt::Formatter,
        state: crate::generator::Uid,
        expected: std::collections::HashSet<crate::grammar::Token>,
    ) -> std::fmt::Result {
        todo!()
    }

    fn visit_goto(
        &self,
        ctx: &super::Ctx,
        f: &mut std::fmt::Formatter,
        symbol: crate::string_pool::Id,
        gotos: &mut dyn Iterator<Item = (crate::generator::Uid, crate::generator::Uid)>,
    ) -> std::fmt::Result {
        todo!()
    }
}
