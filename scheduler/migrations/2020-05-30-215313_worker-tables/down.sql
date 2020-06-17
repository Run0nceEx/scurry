-- This file should undo anything in `up.sql`
use pxdb;

drop table ip_index;
drop table protocols;
drop table providers;
drop table service_index;
drop table service_data;
drop table score_data;
drop index services;

--drop view services;