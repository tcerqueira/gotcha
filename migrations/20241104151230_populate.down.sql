with
  deleted_api_key as (
    delete from public.api_key
    where
      key = '4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q'
    returning
      config
  ),
  deleted_configuration as (
    delete from public.configuration
    where
      id = (
        select
          config
        from
          deleted_api_key
      )
      returning id
  )
select
  *
from
  deleted_configuration;
