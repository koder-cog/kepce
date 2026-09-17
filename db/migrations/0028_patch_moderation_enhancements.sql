-- Moderasyon İyileştirmeleri: İletişim kaynak ayrımı (Kepçe vs Kepçe Ara) ve yanıt geçmişi
ALTER TABLE contact_messages ADD COLUMN IF NOT EXISTS source VARCHAR(50) NOT NULL DEFAULT 'kepce';
ALTER TABLE contact_messages ADD COLUMN IF NOT EXISTS page_url TEXT;
ALTER TABLE contact_messages ADD COLUMN IF NOT EXISTS user_agent TEXT;

CREATE TABLE IF NOT EXISTS contact_message_replies (
    id SERIAL PRIMARY KEY,
    contact_message_id INTEGER NOT NULL REFERENCES contact_messages(id) ON DELETE CASCADE,
    responder_id UUID REFERENCES users(id) ON DELETE SET NULL,
    reply_body TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_contact_message_replies_msg_id ON contact_message_replies(contact_message_id);
