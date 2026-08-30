/// A domain rule stated as a yes-or-no question about one candidate.
///
/// An implementor judges a single candidate and never inspects a collection, so
/// a rule stays meaningful on its own and combines with other rules by ordinary
/// boolean logic. Ranking candidates against each other is a policy's job: a
/// rule that must compare two candidates to answer is not a specification.
pub trait Specification<Candidate> {
    /// Whether `candidate` satisfies this rule.
    fn is_satisfied_by(&self, candidate: &Candidate) -> bool;
}
