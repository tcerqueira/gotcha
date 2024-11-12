create table public.console (
    id uuid not null default gen_random_uuid (),
    label character varying null,
    created_at timestamp with time zone not null default now(),
    constraint console_pkey primary key (id)
);

create table public.api_secret (
    key character varying not null,
    console_id uuid not null,
    encoding_key character varying not null,
    created_at timestamp with time zone not null default now(),
    constraint api_secret_pkey primary key (key),
    constraint api_secret_console_id_fkey foreign key (console_id) references console (id)
        on delete cascade
);
