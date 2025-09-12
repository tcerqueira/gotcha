ALTER TABLE public.api_key ADD COLUMN allowed_domains TEXT[] NOT NULL DEFAULT '{}';
