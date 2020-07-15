-- This file should undo anything in `up.sql`
use pxdb;

drop table Providers;
drop table ProtocolID;
drop table ProtocolProbe;

drop table ServiceIndex;
drop index services;

drop table ServiceData;

drop table ScoreData;
drop table PortFrequencyGraph;
drop table Latency;
drop table SpeedTest;
