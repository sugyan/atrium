use super::super::decision::ModerationDecision;
use super::super::mutewords::has_muted_word;
use super::super::types::{LabelTarget, SubjectPost};
use super::super::Moderator;
use atrium_api::app::bsky::actor::defs::MutedWord;
use atrium_api::app::bsky::embed::record::{ViewBlocked, ViewRecord, ViewRecordRefs};
use atrium_api::app::bsky::embed::record_with_media::{MainMediaRefs, ViewMediaRefs};
use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
use atrium_api::app::bsky::feed::post::{self, RecordEmbedRefs};
use atrium_api::types::{TryFromUnknown, Union};

impl Moderator {
    pub(crate) fn decide_post(&self, subject: &SubjectPost) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        let is_me = self.user_did.as_ref() == Some(&subject.author.did);
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(is_me);
        if let Some(labels) = &subject.labels {
            for label in labels {
                acc.add_label(LabelTarget::Content, label, self);
            }
        }
        if check_hidden_post(subject, &self.prefs.hidden_posts) {
            acc.add_hidden();
        }
        if !is_me && check_muted_words(subject, &self.prefs.muted_words) {
            acc.add_muted_word();
        }

        let embed_acc = match &subject.embed {
            Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(view))) => {
                match &view.record {
                    Union::Refs(ViewRecordRefs::ViewRecord(record)) => {
                        // quoted post
                        Some(self.decide_quoted_post(record))
                    }
                    Union::Refs(ViewRecordRefs::ViewBlocked(blocked)) => {
                        // blocked quote post
                        Some(self.decide_bloked_quoted_post(blocked))
                    }
                    _ => None,
                }
            }
            Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(view))) => {
                match &view.record.record {
                    Union::Refs(ViewRecordRefs::ViewRecord(record)) => {
                        // quoted post with media
                        Some(self.decide_quoted_post(record))
                    }
                    Union::Refs(ViewRecordRefs::ViewBlocked(blocked)) => {
                        // blocked quote post with media
                        Some(self.decide_bloked_quoted_post(blocked))
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        let mut decisions = vec![acc];
        if let Some(mut embed_acc) = embed_acc {
            embed_acc.downgrade();
            decisions.push(embed_acc);
        }
        let author = subject.author.clone().into();
        decisions.extend([self.decide_account(&author), self.decide_profile(&author)]);
        ModerationDecision::merge(&decisions)
    }
    fn decide_quoted_post(&self, subject: &ViewRecord) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(self.user_did.as_ref() == Some(&subject.author.did));
        if let Some(labels) = &subject.labels {
            for label in labels {
                acc.add_label(LabelTarget::Content, label, self);
            }
        }
        ModerationDecision::merge(&[
            acc,
            self.decide_account(&subject.author.clone().into()),
            self.decide_profile(&subject.author.clone().into()),
        ])
    }
    fn decide_bloked_quoted_post(&self, subject: &ViewBlocked) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(self.user_did.as_ref() == Some(&subject.author.did));
        if let Some(viewer) = &subject.author.viewer {
            if viewer.muted.unwrap_or_default() {
                if let Some(list_view) = &viewer.muted_by_list {
                    acc.add_muted_by_list(list_view);
                } else {
                    acc.add_muted();
                }
            }
            if viewer.blocking.is_some() {
                if let Some(list_view) = &viewer.blocking_by_list {
                    acc.add_blocking_by_list(list_view);
                } else {
                    acc.add_blocking();
                }
            }
            if viewer.blocked_by.unwrap_or_default() {
                acc.add_blocked_by();
            }
        }
        acc
    }
}

fn check_hidden_post(subject: &SubjectPost, hidden_posts: &[String]) -> bool {
    if hidden_posts.is_empty() {
        return false;
    }
    if hidden_posts.contains(&subject.uri) {
        return true;
    }
    match &subject.embed {
        Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(view))) => {
            if let Union::Refs(ViewRecordRefs::ViewRecord(record)) = &view.record {
                if hidden_posts.contains(&record.uri) {
                    return true;
                }
            }
        }
        Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(view))) => {
            if let Union::Refs(ViewRecordRefs::ViewRecord(record)) = &view.record.record {
                if hidden_posts.contains(&record.uri) {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

fn check_muted_words(subject: &SubjectPost, muted_words: &[MutedWord]) -> bool {
    if muted_words.is_empty() {
        return false;
    }

    let post_author = &subject.author;

    if let Ok(post) =
        atrium_api::app::bsky::feed::post::Record::try_from_unknown(subject.record.clone())
    {
        // post text
        if has_muted_word(
            muted_words,
            &post.text,
            post.facets.as_ref(),
            post.tags.as_ref(),
            post.langs.as_ref(),
            Some(post_author),
        ) {
            return true;
        }

        if let Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedImagesMain(images))) = &post.embed {
            // post images
            for image in &images.images {
                if has_muted_word(
                    muted_words,
                    &image.alt,
                    None,
                    None,
                    post.langs.as_ref(),
                    Some(post_author),
                ) {
                    return true;
                }
            }
        }
    }

    match &subject.embed {
        // quote post
        Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(view))) => {
            if let Union::Refs(ViewRecordRefs::ViewRecord(view_record)) = &view.record {
                if let Ok(record) = post::Record::try_from_unknown(view_record.value.clone()) {
                    let embedded_post = record;
                    let embed_author = &view_record.author;
                    // quoted post text
                    if has_muted_word(
                        muted_words,
                        &embedded_post.text,
                        embedded_post.facets.as_ref(),
                        embedded_post.tags.as_ref(),
                        embedded_post.langs.as_ref(),
                        Some(embed_author),
                    ) {
                        return true;
                    }
                    match &embedded_post.embed {
                        // quoted post's images
                        Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedImagesMain(main))) => {
                            for image in &main.images {
                                if has_muted_word(
                                    muted_words,
                                    &image.alt,
                                    None,
                                    None,
                                    embedded_post.langs.as_ref(),
                                    Some(embed_author),
                                ) {
                                    return true;
                                }
                            }
                        }
                        // quoted post's link card
                        Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedExternalMain(main))) => {
                            let external = &main.external;
                            if has_muted_word(
                                muted_words,
                                &format!("{} {}", external.title, external.description),
                                None,
                                None,
                                None,
                                Some(embed_author),
                            ) {
                                return true;
                            }
                        }
                        Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedRecordWithMediaMain(
                            main,
                        ))) => match &main.media {
                            Union::Refs(MainMediaRefs::AppBskyEmbedExternalMain(main)) => {
                                let external = &main.external;
                                if has_muted_word(
                                    muted_words,
                                    &format!("{} {}", external.title, external.description),
                                    None,
                                    None,
                                    None,
                                    Some(embed_author),
                                ) {
                                    return true;
                                }
                            }
                            Union::Refs(MainMediaRefs::AppBskyEmbedImagesMain(main)) => {
                                for image in &main.images {
                                    if has_muted_word(
                                        muted_words,
                                        &image.alt,
                                        None,
                                        None,
                                        embedded_post.langs.as_ref(),
                                        Some(embed_author),
                                    ) {
                                        return true;
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
        // link card
        Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedExternalView(view))) => {
            let external = &view.external;
            if has_muted_word(
                muted_words,
                &format!("{} {}", external.title, external.description),
                None,
                None,
                None,
                Some(post_author),
            ) {
                return true;
            }
        }
        // quote post with media
        Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(view))) => {
            if let Union::Refs(ViewRecordRefs::ViewRecord(view_record)) = &view.record.record {
                let embed_author = &view_record.author;
                // quoted post text
                if let Ok(record) = post::Record::try_from_unknown(view_record.value.clone()) {
                    let post = record;
                    if has_muted_word(
                        muted_words,
                        &post.text,
                        post.facets.as_ref(),
                        post.tags.as_ref(),
                        post.langs.as_ref(),
                        Some(embed_author),
                    ) {
                        return true;
                    }
                }
                // quoted post media
                match &view.media {
                    Union::Refs(ViewMediaRefs::AppBskyEmbedExternalView(view)) => {
                        let external = &view.external;
                        if has_muted_word(
                            muted_words,
                            &format!("{} {}", external.title, external.description),
                            None,
                            None,
                            None,
                            Some(embed_author),
                        ) {
                            return true;
                        }
                    }
                    Union::Refs(ViewMediaRefs::AppBskyEmbedImagesView(view)) => {
                        let langs = post::Record::try_from_unknown(view_record.value.clone())
                            .ok()
                            .and_then(|record| record.data.langs);
                        for image in &view.images {
                            if has_muted_word(
                                muted_words,
                                &image.alt,
                                None,
                                None,
                                langs.as_ref(),
                                Some(embed_author),
                            ) {
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    false
}
