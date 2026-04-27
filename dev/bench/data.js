window.BENCHMARK_DATA = {
  "lastUpdate": 1777297714426,
  "repoUrl": "https://github.com/WJQSERVER/gitserver",
  "entries": {
    "git-server Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "af052f4fa5515e25a574ad61f06721cd0b6e0958",
          "message": "Merge pull request #1 from WJQSERVER/feat/v2-fetch-streaming\n\nFeat/v2 fetch streaming",
          "timestamp": "2026-04-07T00:58:54+08:00",
          "tree_id": "55821de9d3ce0c50411bd467222da9ab6bb3d658",
          "url": "https://github.com/WJQSERVER/git-server/commit/af052f4fa5515e25a574ad61f06721cd0b6e0958"
        },
        "date": 1775495814415,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 103671654,
            "range": "± 7998816",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 113696767,
            "range": "± 13438869",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 156399225,
            "range": "± 3488929",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 252698429,
            "range": "± 9113442",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 469877189,
            "range": "± 7157708",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 931289718,
            "range": "± 7558314",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 58556223,
            "range": "± 506664",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 102188081,
            "range": "± 9547852",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1568423940,
            "range": "± 38606933",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85026685,
            "range": "± 6061737",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 121133193,
            "range": "± 10795901",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1469885898,
            "range": "± 4056662",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 946425,
            "range": "± 16047",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 27723806,
            "range": "± 3774584",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1275899718,
            "range": "± 25119206",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 205555,
            "range": "± 2153",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 266538,
            "range": "± 2745",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 348617,
            "range": "± 2578",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4e9112971036b08122bdda8adf74ac9ae2ecf03c",
          "message": "Merge pull request #4 from WJQSERVER/fix/followup-cleanups\n\nFix/followup cleanups",
          "timestamp": "2026-04-07T03:09:07+08:00",
          "tree_id": "a446a62aa0af29c65d7ef65031fc41085824c70b",
          "url": "https://github.com/WJQSERVER/git-server/commit/4e9112971036b08122bdda8adf74ac9ae2ecf03c"
        },
        "date": 1775503508713,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 102553742,
            "range": "± 10281381",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 113806622,
            "range": "± 6877141",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 158502471,
            "range": "± 4784173",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 255216125,
            "range": "± 8098928",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 471612722,
            "range": "± 4784204",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 936788788,
            "range": "± 21971943",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 58569926,
            "range": "± 1172530",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 103085262,
            "range": "± 6863561",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1547576370,
            "range": "± 26469722",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85301529,
            "range": "± 10951177",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 120829015,
            "range": "± 7887664",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1466902747,
            "range": "± 4175312",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 916315,
            "range": "± 17189",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 29250238,
            "range": "± 3044803",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1271766968,
            "range": "± 18687752",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 204475,
            "range": "± 1429",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 266082,
            "range": "± 2340",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 352489,
            "range": "± 2522",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9460fbeec82cd8c550063c43c13497f941c402f0",
          "message": "Merge pull request #7 from WJQSERVER/dependabot/cargo/rustls-webpki-0.103.10\n\nbuild(deps): bump rustls-webpki from 0.103.9 to 0.103.10",
          "timestamp": "2026-04-08T05:30:20+08:00",
          "tree_id": "7789052f75c80dc1e09928c35857dbe1d36a2307",
          "url": "https://github.com/WJQSERVER/gitserver/commit/9460fbeec82cd8c550063c43c13497f941c402f0"
        },
        "date": 1775598865406,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 113369244,
            "range": "± 8234688",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 154891307,
            "range": "± 40925421",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 251654473,
            "range": "± 9062780",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 455761740,
            "range": "± 7786296",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 892775107,
            "range": "± 15636710",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1808527257,
            "range": "± 25080703",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 60430190,
            "range": "± 1051551",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 112929969,
            "range": "± 8079311",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1796621381,
            "range": "± 23249525",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85985505,
            "range": "± 1215472",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 125594630,
            "range": "± 13299740",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1481277861,
            "range": "± 14962751",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1099649,
            "range": "± 25241",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 36462999,
            "range": "± 1446475",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1354406821,
            "range": "± 7792131",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 207981,
            "range": "± 3250",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 270732,
            "range": "± 12013",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 355868,
            "range": "± 3640",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "gitserver Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fb05600cef4201e799e833465c7403f1ed4d4830",
          "message": "Merge pull request #9 from WJQSERVER/docs\n\nDocs",
          "timestamp": "2026-04-09T03:19:07+08:00",
          "tree_id": "08eb7af359ffc4f8f2908b69d2deb8a803d82c83",
          "url": "https://github.com/WJQSERVER/gitserver/commit/fb05600cef4201e799e833465c7403f1ed4d4830"
        },
        "date": 1775677268310,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 98954022,
            "range": "± 2027564",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 135090705,
            "range": "± 53864942",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 221335545,
            "range": "± 6651404",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 387901692,
            "range": "± 6307459",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 772316297,
            "range": "± 7041288",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1546975452,
            "range": "± 10900336",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 54857911,
            "range": "± 441041",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 98305982,
            "range": "± 4082734",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1639947815,
            "range": "± 29989835",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 84292180,
            "range": "± 20146238",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 116182858,
            "range": "± 10739969",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1275785126,
            "range": "± 16119505",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 753643,
            "range": "± 15344",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 28866000,
            "range": "± 2456261",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1152486373,
            "range": "± 12441165",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 118617,
            "range": "± 1298",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 159954,
            "range": "± 12521",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 207516,
            "range": "± 3548",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "566ef2310f6cc1939eb8876a8b3efcf1817d6ab8",
          "message": "fix: use CARGO_PKG_VERSION for agent version (#10)\n\n* fix: use CARGO_PKG_VERSION for agent version in capabilities\n\n* fix: revert version to 0.0.1\n\n* chore: update Cargo.lock for version 0.0.1\n\n* style: format code with rustfmt",
          "timestamp": "2026-04-09T04:22:18+08:00",
          "tree_id": "9f13a7f803a7167d5ff4296512f5363788180c2b",
          "url": "https://github.com/WJQSERVER/gitserver/commit/566ef2310f6cc1939eb8876a8b3efcf1817d6ab8"
        },
        "date": 1775680973666,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 113444125,
            "range": "± 7034199",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 154970552,
            "range": "± 25070910",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 252101061,
            "range": "± 9671489",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 455106545,
            "range": "± 10808171",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 902852425,
            "range": "± 16927338",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1816980275,
            "range": "± 29108938",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 61337325,
            "range": "± 1853716",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 113414142,
            "range": "± 6413066",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1798357236,
            "range": "± 31198236",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 86997874,
            "range": "± 4280340",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 126012590,
            "range": "± 16452114",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1485018983,
            "range": "± 9865821",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1118191,
            "range": "± 46305",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 36544833,
            "range": "± 1523210",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1356663018,
            "range": "± 7456256",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 207852,
            "range": "± 4771",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 270394,
            "range": "± 20189",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 357662,
            "range": "± 27223",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "wjqserver",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "wjqserver",
            "username": "WJQSERVER"
          },
          "distinct": true,
          "id": "601132e4bfce33c8431ae09db8b533a703c0e9ba",
          "message": "rename: LICENSE-UPSTREAM-MIT to UPSTREAM-LICENSE",
          "timestamp": "2026-04-09T04:23:53+08:00",
          "tree_id": "06ec8d52dd61e0d5c2ea03065a976e3cc5c364d1",
          "url": "https://github.com/WJQSERVER/gitserver/commit/601132e4bfce33c8431ae09db8b533a703c0e9ba"
        },
        "date": 1775681095972,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 115881757,
            "range": "± 5056791",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 157441535,
            "range": "± 31278130",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 255525890,
            "range": "± 7284559",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 466648218,
            "range": "± 10745072",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 919352097,
            "range": "± 14061871",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1843995647,
            "range": "± 22447905",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 60257208,
            "range": "± 2825756",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 113619389,
            "range": "± 11369593",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1879025035,
            "range": "± 19859813",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 82977022,
            "range": "± 14556011",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 124846626,
            "range": "± 12506515",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1571225990,
            "range": "± 9535575",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1140348,
            "range": "± 32064",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 38602596,
            "range": "± 1955506",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1446144108,
            "range": "± 6956345",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 213406,
            "range": "± 2428",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 275104,
            "range": "± 4611",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 369452,
            "range": "± 18198",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0f967cef6fa174db7e24fa76539491fc79d5e641",
          "message": "update NOTICE file",
          "timestamp": "2026-04-09T04:33:50+08:00",
          "tree_id": "d314aa7781628272f3cf757e2a611e8f6360943f",
          "url": "https://github.com/WJQSERVER/gitserver/commit/0f967cef6fa174db7e24fa76539491fc79d5e641"
        },
        "date": 1775681634870,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 111618879,
            "range": "± 7878219",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 153493937,
            "range": "± 32365723",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 247485667,
            "range": "± 7452315",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 445335966,
            "range": "± 11395575",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 885225101,
            "range": "± 17556767",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1751687198,
            "range": "± 20255529",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 59111781,
            "range": "± 1390317",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 109923738,
            "range": "± 7234864",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1773803083,
            "range": "± 24439643",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 87081410,
            "range": "± 4256103",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 124676994,
            "range": "± 14685563",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1466715593,
            "range": "± 10420221",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1035499,
            "range": "± 26645",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 34395493,
            "range": "± 1038414",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1329187202,
            "range": "± 6823918",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 206572,
            "range": "± 12109",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 270559,
            "range": "± 5813",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 354990,
            "range": "± 2310",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c2f9fcd76166e81ec3ea5555c4ecf856d1c5fb3b",
          "message": "Graceful shutdown with draining and hardening (#11)\n\n* feat: add graceful shutdown draining\n\nKeep in-flight Git transfers running while readiness flips to draining during shutdown.\n\n* fix: harden graceful shutdown draining\n\nReject new Git work as soon as draining starts and avoid shutting the server down when signal handler setup fails.\n\n* refactor: tighten shutdown review followups\n\nExtract the shared shutdown message, remove the redundant upload-pack draining check, and document subtle shutdown lifecycle edge cases.\n\n* fix: publish draining state across workers\n\nUse release/acquire ordering for the shutdown flag while keeping request-path checks as a single atomic read.",
          "timestamp": "2026-04-10T20:26:07+08:00",
          "tree_id": "90add680a169eb979812d80bdd06d1b78588af6c",
          "url": "https://github.com/WJQSERVER/gitserver/commit/c2f9fcd76166e81ec3ea5555c4ecf856d1c5fb3b"
        },
        "date": 1775825219366,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 114162123,
            "range": "± 8369611",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 154616926,
            "range": "± 36274266",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 253180431,
            "range": "± 8141390",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 455204233,
            "range": "± 6826968",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 904827082,
            "range": "± 8807370",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1808930804,
            "range": "± 28479751",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 59992354,
            "range": "± 1173960",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 113614383,
            "range": "± 6632154",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1875184788,
            "range": "± 23251403",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 82064404,
            "range": "± 5592491",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 123316763,
            "range": "± 10231021",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1560145193,
            "range": "± 6353587",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1130053,
            "range": "± 33545",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 38415169,
            "range": "± 2115090",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1453736040,
            "range": "± 12422789",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 210323,
            "range": "± 2258",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 270950,
            "range": "± 5168",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 365506,
            "range": "± 4999",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a4872837f508677e9322048cd0ae66597b5008b0",
          "message": "Merge pull request #12 from WJQSERVER/feat/graceful-shutdown\n\nci: switch release publishing to trusted publishing",
          "timestamp": "2026-04-11T00:10:14+08:00",
          "tree_id": "f20dd910ea5cd06445a48214f701050eee3eb021",
          "url": "https://github.com/WJQSERVER/gitserver/commit/a4872837f508677e9322048cd0ae66597b5008b0"
        },
        "date": 1775838623392,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 111736395,
            "range": "± 5795621",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 151543441,
            "range": "± 32397159",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 247512670,
            "range": "± 9267596",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 443983724,
            "range": "± 7285584",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 880146368,
            "range": "± 14635187",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1773670916,
            "range": "± 17451162",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 59252068,
            "range": "± 4686055",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 111305236,
            "range": "± 4561589",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1783457497,
            "range": "± 25134149",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 86778605,
            "range": "± 854807",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 125315592,
            "range": "± 13820371",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1497009356,
            "range": "± 19517153",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1077379,
            "range": "± 43028",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 35329301,
            "range": "± 1685924",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1387302956,
            "range": "± 5960442",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 206060,
            "range": "± 4014",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 270349,
            "range": "± 14544",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 356213,
            "range": "± 3949",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a4bdc491a5c806bdbbd817a2b765095b7109be8c",
          "message": "chore: prepare v0.0.2 release (#13)\n\nBump the workspace and published crate versions to 0.0.2 and refresh the lockfile for the release tag.",
          "timestamp": "2026-04-11T00:26:32+08:00",
          "tree_id": "582057e5d83614c8fa81faf158463aca0c09586d",
          "url": "https://github.com/WJQSERVER/gitserver/commit/a4bdc491a5c806bdbbd817a2b765095b7109be8c"
        },
        "date": 1775839612963,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 113734237,
            "range": "± 7015503",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 155175582,
            "range": "± 38952617",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 251927783,
            "range": "± 8440238",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 453115789,
            "range": "± 8498952",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 889929251,
            "range": "± 17198413",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1779390947,
            "range": "± 30044887",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 59950122,
            "range": "± 11068051",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 113664159,
            "range": "± 9460544",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1784794331,
            "range": "± 20232102",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85231763,
            "range": "± 6977636",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 124009207,
            "range": "± 15125199",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1478885263,
            "range": "± 14500677",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1117013,
            "range": "± 24494",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 36047233,
            "range": "± 1319463",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1353652635,
            "range": "± 22703714",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 204675,
            "range": "± 4221",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 267923,
            "range": "± 4073",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 350183,
            "range": "± 2342",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fdca0993692821156ab50db325275f7ca06b6704",
          "message": "fix: avoid curl-based crates.io version probes (#14)\n\nUse cargo info outside the workspace to detect published crate versions without tripping crates.io API 403 responses in CI.",
          "timestamp": "2026-04-11T00:48:38+08:00",
          "tree_id": "26a39559c7db31624ccd8ad9734a543e1136d87f",
          "url": "https://github.com/WJQSERVER/gitserver/commit/fdca0993692821156ab50db325275f7ca06b6704"
        },
        "date": 1775840682982,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 101821483,
            "range": "± 74268669",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 113700623,
            "range": "± 5698874",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 156001146,
            "range": "± 3400456",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 251449855,
            "range": "± 8295444",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 465757094,
            "range": "± 5676027",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 923513832,
            "range": "± 8438064",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 58024018,
            "range": "± 560941",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 101806682,
            "range": "± 9692196",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1555166729,
            "range": "± 23736410",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85078259,
            "range": "± 10932651",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 121164834,
            "range": "± 10158617",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1476220229,
            "range": "± 7741206",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1012019,
            "range": "± 61850",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 28272390,
            "range": "± 3109918",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1305633155,
            "range": "± 23090051",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 204927,
            "range": "± 2684",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 267800,
            "range": "± 2323",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 353086,
            "range": "± 2533",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "wjqserver",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "wjqserver",
            "username": "WJQSERVER"
          },
          "distinct": false,
          "id": "f6cbc73318b0ffd680dd87944646012859eccdbb",
          "message": "chore: prepare v0.0.3 release\n\nBump the workspace and published crate versions to 0.0.3 and refresh the lockfile for the next release tag.",
          "timestamp": "2026-04-11T00:58:16+08:00",
          "tree_id": "d1dbaa2e62d5280da7155f67a676486805f56b2e",
          "url": "https://github.com/WJQSERVER/gitserver/commit/f6cbc73318b0ffd680dd87944646012859eccdbb"
        },
        "date": 1775841259422,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 102468938,
            "range": "± 10323789",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 113574442,
            "range": "± 9064075",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 157186639,
            "range": "± 3978577",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 255721682,
            "range": "± 8396533",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 470743749,
            "range": "± 5409411",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 938440260,
            "range": "± 8205703",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 58149924,
            "range": "± 1736824",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 102948010,
            "range": "± 8817007",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1577827911,
            "range": "± 26434830",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85758047,
            "range": "± 10195550",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 120844197,
            "range": "± 7823779",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1481747330,
            "range": "± 3866952",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 921536,
            "range": "± 18392",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 27795385,
            "range": "± 2589111",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1279245337,
            "range": "± 18154121",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 205979,
            "range": "± 1378",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 267310,
            "range": "± 2517",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 354375,
            "range": "± 2438",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3cec05b02eea3220c67782e4d9d0404d8b001aa3",
          "message": "build(deps): bump openssl from 0.10.76 to 0.10.78 (#17)\n\nBumps [openssl](https://github.com/rust-openssl/rust-openssl) from 0.10.76 to 0.10.78.\n- [Release notes](https://github.com/rust-openssl/rust-openssl/releases)\n- [Commits](https://github.com/rust-openssl/rust-openssl/compare/openssl-v0.10.76...openssl-v0.10.78)\n\n---\nupdated-dependencies:\n- dependency-name: openssl\n  dependency-version: 0.10.78\n  dependency-type: indirect\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-27T21:30:24+08:00",
          "tree_id": "f703bead63debe14ea917fbc22f748d1a3122c8b",
          "url": "https://github.com/WJQSERVER/gitserver/commit/3cec05b02eea3220c67782e4d9d0404d8b001aa3"
        },
        "date": 1777297713578,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 105109279,
            "range": "± 12581463",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 115009794,
            "range": "± 4762476",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 157264028,
            "range": "± 4054849",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 255820001,
            "range": "± 7123971",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 472781661,
            "range": "± 6919190",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 939255816,
            "range": "± 8953698",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 59409500,
            "range": "± 1992513",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 104110036,
            "range": "± 8180529",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1590471979,
            "range": "± 26394950",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 83915959,
            "range": "± 12968375",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 121009998,
            "range": "± 10177686",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1482635926,
            "range": "± 4875063",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 983561,
            "range": "± 27909",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 31879492,
            "range": "± 3781067",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1264090750,
            "range": "± 28525974",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 205396,
            "range": "± 2739",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 266585,
            "range": "± 2636",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 352938,
            "range": "± 2370",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}