use ruff_text_size::TextRange;
use ty_semantic_index::{
    BindingWithConstraints, FileScopeId, SemanticIndex, Truthiness, UseDefMap,
    reachability_constraints::ScopedReachabilityConstraintId,
};

use crate::{
    Db, place::evaluate_reachability, reachability_constraints::evaluate_reachability_constraint,
};

/// Check whether a diagnostic emitted at `range` is in reachable code, considering both
/// scope reachability and statement-level reachability within the scope.
pub(crate) fn is_range_reachable<'db>(
    db: &'db dyn crate::Db,
    index: &SemanticIndex<'db>,
    scope_id: FileScopeId,
    range: TextRange,
) -> bool {
    let use_def = index.use_def_map(scope_id);
    is_scope_reachable(db, index, scope_id)
        && !index.use_def_map(scope_id).range_reachability().iter().any(
            |&(entry_range, constraint)| {
                entry_range.contains_range(range) && !is_reachable(db, use_def, constraint)
            },
        )
}

pub(crate) fn is_reachable<'db>(
    db: &'db dyn Db,
    use_def: &UseDefMap<'db>,
    reachability: ScopedReachabilityConstraintId,
) -> bool {
    evaluate_reachability(db, use_def, reachability).may_be_true()
}

pub(crate) fn is_scope_reachable<'db>(
    db: &'db dyn Db,
    index: &SemanticIndex<'db>,
    scope_id: FileScopeId,
) -> bool {
    index
        .parent_scope_id(scope_id)
        .is_none_or(|parent_scope_id| {
            if !is_scope_reachable(db, index, parent_scope_id) {
                return false;
            }

            let parent_use_def = index.use_def_map(parent_scope_id);
            let reachability = index.scope(scope_id).reachability();

            is_reachable(db, parent_use_def, reachability)
        })
}

pub(crate) fn binding_reachability<'db, 'map>(
    db: &'db dyn Db,
    use_def: &'map UseDefMap<'db>,
    binding: &BindingWithConstraints<'map, 'db>,
) -> Truthiness {
    evaluate_reachability_constraint(
        db,
        use_def.reachability_constraints(),
        use_def.predicates(),
        binding.reachability_constraint,
    )
}
