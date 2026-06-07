use api_types::{
    auth::UserResponse,
    notes::{NoteLinkResponse, NoteResponse, NoteVersionResponse},
    tags::TagResponse,
};
use domain::{
    note::entity::{Note, NoteLink, NoteVersion},
    tag::entity::Tag,
    user::entity::User,
};

pub fn tag_response(t: Tag) -> TagResponse {
    TagResponse {
        id: t.id.as_uuid(),
        name: t.name.into_inner(),
    }
}

pub fn note_response(n: Note) -> NoteResponse {
    NoteResponse {
        id: n.id.as_uuid(),
        user_id: n.user_id.as_uuid(),
        title: n.title.map(|t| t.into_inner()),
        content: n.content,
        color: n.color.into_inner(),
        is_pinned: n.is_pinned,
        is_archived: n.is_archived,
        created_at: n.created_at,
        updated_at: n.updated_at,
        tags: n.tags.into_iter().map(tag_response).collect(),
    }
}

pub fn note_version_response(v: NoteVersion) -> NoteVersionResponse {
    NoteVersionResponse {
        id: v.id,
        note_id: v.note_id.as_uuid(),
        title: v.title,
        content: v.content,
        created_at: v.created_at,
    }
}

pub fn note_link_response(l: NoteLink) -> NoteLinkResponse {
    NoteLinkResponse {
        source_id: l.source_id.as_uuid(),
        target_id: l.target_id.as_uuid(),
        score: l.score,
        created_at: l.created_at,
    }
}

pub fn user_response(u: User) -> UserResponse {
    UserResponse {
        id: u.id.as_uuid(),
        email: u.email.into_inner(),
        created_at: u.created_at,
    }
}
