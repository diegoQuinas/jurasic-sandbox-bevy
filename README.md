
# Dinosaur simulator


## Roadmap


- [ ] 16 systems on chain(), nothing runs in parallel (tons of queries)
- [ ] render -> HashMap of ALL entities and redraws the terminal on each Update
- [ ] PopulationHistory grows one point on tick without top. Ratatui chart gets heavy
- [ ] find_closest_tile Vec::with_capacity(3) per movement
- [ ] Not scanning all plants, maybe use chunks "plant near in radio N"
- [ ] Filter queries, only SeekingFood / SeekingPartner
- [ ] Limit catch-up of Fixed
- [ ] Cap or downsample the history of charts
