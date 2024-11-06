with
  config as (insert into public.configuration (LABEL) values ('test') returning id)
insert into
  public.api_secret (key, config, encoding_key)
values
  (
    '4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q',
    (select id from config),
    '4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q'
  );
