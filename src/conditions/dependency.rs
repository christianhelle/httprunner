use crate::types::RequestContext;

pub fn check_dependency(depends_on: &Option<String>, context: &[RequestContext]) -> bool {
    if let Some(dep_name) = depends_on {
        let target_context = context.iter().find(|ctx| ctx.name == *dep_name);

        if let Some(ctx) = target_context
            && let Some(ref result) = ctx.result
        {
            return result.success;
        }
        return false;
    }

    true
}
