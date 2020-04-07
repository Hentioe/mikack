use mikack::extractors::*;

#[allow(unused_macros)]
macro_rules! assert_routes {
    ($domain:expr, :comic => $comic_url:expr, :chapter => $chapter_url:expr) => {
        assert_eq!(
            Some(DomainRoute::Comic(String::from($domain))),
            domain_route($comic_url)
        );
        assert_eq!(
            DomainRoute::Chapter(String::from($domain)),
            domain_route($chapter_url).unwrap()
        );
    };
    ($domain:expr, :chapter => $chapter_url:expr) => {
        assert_eq!(
            DomainRoute::Chapter(String::from($domain)),
            domain_route($chapter_url).unwrap()
        );
    };
    ($domain:expr, :comic => $comic_url:expr) => {
        assert_eq!(
            DomainRoute::Comic(String::from($domain)),
            domain_route($comic_url).unwrap()
        );
    };
}

#[test]
fn test_routes() {
    assert_routes!("www.bidongmh.com",
        :comic   => "https://www.bidongmh.com/book/256",
        :chapter => "https://www.bidongmh.com/chapter/6807"
    );
    assert_routes!("www.bnmanhua.com",
        :comic   => "https://www.bnmanhua.com/comic/15195.html",
        :chapter => "https://www.bnmanhua.com/comic/15195/421378.html"
    );
    assert_routes!("www.cartoonmad.com",
        :comic   => "https://www.cartoonmad.com/comic/8460.html",
        :chapter => "https://www.cartoonmad.com/comic/846000012038001.html"
    );
    assert_routes!("www.comico.com.tw",
        :comic   => "http://www.comico.com.tw/challenge/3711/",
        :chapter => "http://www.comico.com.tw/challenge/3711/1/"
    );
    assert_routes!("www.dm5.com",
        :comic   => "http://www.dm5.com/manhua-yuanzun/",
        :chapter => "http://www.dm5.com/m578500/"
    );
    assert_routes!("manhua.dmzj.com",
        :comic   => "http://manhua.dmzj.com/yifuyaozhemechuan/",
        :chapter => "http://manhua.dmzj.com/yifuyaozhemechuan/56275.shtml"
    );
    assert_routes!("e-hentai.org",
        :chapter => "https://e-hentai.org/g/1552929/c9f7a6ad71/"
    );
    assert_routes!("18h.animezilla.com",
        :chapter => "https://18h.animezilla.com/manga/2940"
    );
    assert_routes!("www.gufengmh8.com",
        :comic   => "https://www.gufengmh8.com/manhua/dongjingshishiguire/",
        :chapter => "https://www.gufengmh8.com/manhua/dongjingshishiguire/8519.html"
    );
    assert_routes!("c-upp.com",
        :chapter => "https://c-upp.com/ja/s/315668/"
    );
    assert_routes!("www.hhimm.com",
        :comic   => "http://www.hhimm.com/manhua/40325.html",
        :chapter => "http://www.hhimm.com/cool373925/1.html?s=3"
    );
    assert_routes!("www.pufei8.com",
        :comic   => "http://www.pufei8.com/manhua/600/index.html",
        :chapter => "http://www.pufei8.com/manhua/600/45661.html"
    );
    assert_routes!("www.kuaikanmanhua.com",
        :comic   => "https://www.kuaikanmanhua.com/web/topic/544/",
        :chapter => "https://www.kuaikanmanhua.com/web/comic/5471/"
    );
    assert_routes!("comic.ikkdm.com",
        :comic   => "http://comic.ikkdm.com/comiclist/2555/index.htm",
        :chapter => "http://comic.ikkdm.com/comiclist/2555/66929/1.htm"
    );
    assert_routes!("loveheaven.net",
        :comic   => "https://loveheaven.net/manga-ichinichi-gaishutsuroku-hanchou-raw.html",
        :chapter => "https://loveheaven.net/read-ichinichi-gaishutsuroku-hanchou-raw-chapter-54.html"
    );
    assert_routes!("www.luscious.net",
        :chapter => "https://www.luscious.net/albums/teitoku-wa-semai-toko-suki-kantai-collection-kanco_363520/"
    );
    assert_routes!("www.mangabz.com",
        :comic   => "http://www.mangabz.com/565bz/",
        :chapter => "http://www.mangabz.com/m93502/"
    );
    assert_routes!("manganelo.com",
        :comic   => "https://manganelo.com/manga/hgj2047065412",
        :chapter => "https://manganelo.com/chapter/hgj2047065412/chapter_43"
    );
    assert_routes!("www.manhuadb.com",
        :comic   => "https://www.manhuadb.com/manhua/10906",
        :chapter => "https://www.manhuadb.com/manhua/10906/13071_183254.html"
    );
    assert_routes!("www.manhuadui.com",
        :comic   => "https://www.manhuadui.com/manhua/jingjiechufazhe/",
        :chapter => "https://www.manhuadui.com/manhua/jingjiechufazhe/435634.html"
    );
    assert_routes!("www.manhuagui.com",
        :comic   => "https://www.manhuagui.com/comic/20515/",
        :chapter => "https://www.manhuagui.com/comic/20515/469245.html"
    );
    assert_routes!("www.manhuapu.com",
        :comic   => "http://www.manhuapu.com/rexue/xiaxingjiutian/",
        :chapter => "http://www.manhuapu.com/rexue/xiaxingjiutian/719652.html"
    );
    assert_routes!("nhentai.net",
        :chapter => "https://nhentai.net/g/300773/"
    );
    assert_routes!("9hentai.com",
        :chapter => "https://9hentai.com/g/60726/"
    );
    assert_routes!("www.90mh.com",
        :comic   => "http://www.90mh.com/manhua/taguoriji/",
        :chapter => "http://www.90mh.com/manhua/taguoriji/127184.html"
    );
    assert_routes!("www.177pic.info",
        :chapter => "http://www.177pic.info/html/2020/01/3307768.html"
    );
    assert_routes!("www.onemanhua.com",
        :comic   => "https://www.onemanhua.com/12436/",
        :chapter => "https://www.onemanhua.com/12436/1/1.html"
    );
    assert_routes!("www.qimiaomh.com",
        :comic   => "https://www.qimiaomh.com/manhua/6531.html",
        :chapter => "https://www.qimiaomh.com/manhua/6531/1.html"
    );
    assert_routes!("www.tohomh123.com",
        :comic   => "https://www.tohomh123.com/guangzhizi/",
        :chapter => "https://www.tohomh123.com/guangzhizi/1.html"
    );
    assert_routes!("twhentai.com",
        :chapter => "http://twhentai.com/hentai_doujin/68098/"
    );
    assert_routes!("www.2animx.com",
        :comic   => "http://www.2animx.com/index-comic-name-%E9%A2%A8%E9%9B%B2%E5%85%A8%E9%9B%86-id-7212",
        :chapter => "http://www.2animx.com/index-look-name-%E9%A2%A8%E9%9B%B2%E5%85%A8%E9%9B%86-cid-7212-id-88034"
    );
    assert_routes!("www.wnacg.org",
        :chapter => "https://www.wnacg.org/photos-index-aid-94352.html"
    );
    assert_routes!("www.wuqimh.com",
        :comic   => "http://www.wuqimh.com/38157/",
        :chapter => "http://www.wuqimh.com/38157/01.html"
    );
    assert_routes!("www.177mh.net",
        :comic   => "https://www.177mh.net/colist_244241.html",
        :chapter => "https://www.177mh.net/202001/437290.html"
    );
    assert_routes!("8comic.se",
        :chapter => "http://8comic.se/879/"
    );
}
