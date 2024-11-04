create table public.configuration (
    id uuid not null default gen_random_uuid (),
    label character varying null,
    created_at timestamp with time zone not null default now(),
    constraint configuration_pkey primary key (id)
);

create table public.api_key (
    key character varying not null,
    config uuid not null,
    created_at timestamp with time zone not null default now(),
    constraint api_key_pkey primary key (key),
    constraint api_key_config_fkey foreign key (config) references configuration (id)
);

create table public.active_challenge (
    id uuid not null default gen_random_uuid (),
    api_key character varying not null,
    encoding_key character varying not null,
    ip_addr inet not null,
    created_at timestamp with time zone not null default now(),
    constraint active_challenge_pkey primary key (id),
    constraint active_challenge_api_key_fkey foreign key (api_key) references api_key (key)
);
