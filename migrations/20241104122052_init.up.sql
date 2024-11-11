create table public.configuration (
    id uuid not null default gen_random_uuid (),
    label character varying null,
    created_at timestamp with time zone not null default now(),
    constraint configuration_pkey primary key (id)
);

create table public.api_secret (
    key character varying not null,
    config uuid not null,
    encoding_key character varying not null,
    created_at timestamp with time zone not null default now(),
    constraint api_secret_pkey primary key (key),
    constraint api_secret_config_fkey foreign key (config) references configuration (id)
        on delete cascade
);
