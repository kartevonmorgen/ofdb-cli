use ofdb_boundary::Review;
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};
use uuid::Uuid;

pub fn group_reviews(reviews: Vec<(Uuid, Review)>) -> Vec<(Review, HashSet<Uuid>)> {
    let mut groups = HashMap::new();
    for (uuid, rev) in reviews {
        let uuids = groups.entry(Rev::from(rev)).or_insert_with(HashSet::new);
        uuids.insert(uuid);
    }
    groups
        .into_iter()
        .map(|(rev, ids)| (Review::from(rev), ids))
        .collect()
}

// Workaround:
// because `Review` does not implement `PartialEq`, `Eq` and `Hash`.
struct Rev(Review);

impl Hash for Rev {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.status.hash(state);
        self.0.comment.hash(state);
    }
}

impl PartialEq for Rev {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.status == rhs.0.status && self.0.comment == rhs.0.comment
    }
}

impl Eq for Rev {}

impl From<Review> for Rev {
    fn from(r: Review) -> Self {
        Self(r)
    }
}

impl From<Rev> for Review {
    fn from(r: Rev) -> Self {
        r.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ofdb_boundary::ReviewStatus;

    #[test]
    fn group_uuids_by_review() {
        let uuid_0 = Uuid::new_v4();
        let uuid_1 = Uuid::new_v4();
        let uuid_2 = Uuid::new_v4();
        let uuid_3 = Uuid::new_v4();

        let rev_0 = Review {
            status: ReviewStatus::Archived,
            comment: Some("foo".into()),
        };
        let rev_1 = Review {
            status: ReviewStatus::Archived,
            comment: Some("foo".into()),
        };
        let rev_2 = Review {
            status: ReviewStatus::Archived,
            comment: None,
        };
        let rev_3 = Review {
            status: ReviewStatus::Created,
            comment: None,
        };

        let reviews = vec![
            (uuid_0, rev_0.clone()),
            (uuid_1, rev_1.clone()),
            (uuid_2, rev_2.clone()),
            (uuid_3, rev_3.clone()),
        ];

        let groups = group_reviews(reviews);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups.iter().map(|(_, ids)| ids.len()).max().unwrap(), 2);
        assert_eq!(groups.iter().map(|(_, ids)| ids.len()).min().unwrap(), 1);

        for (rev, ids) in groups {
            if ids.len() == 2 {
                assert!(ids.contains(&uuid_0));
                assert!(ids.contains(&uuid_1));
            }
            if rev.comment == rev_2.comment && rev.status == rev_2.status {
                assert_eq!(ids.len(), 1);
                assert!(ids.contains(&uuid_2));
            }
            if rev.comment == rev_3.comment && rev.status == rev_3.status {
                assert_eq!(ids.len(), 1);
                assert!(ids.contains(&uuid_3));
            }
        }
    }
}
