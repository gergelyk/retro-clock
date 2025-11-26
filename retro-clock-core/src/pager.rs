use crate::http_client::HttpClient;
use crate::semaphore::Semaphore;
use crate::terminal::Terminal;
use anyhow::Context;
use std::sync::Arc;

pub trait Page<T: Terminal, H: HttpClient> {
    fn render_title(&self, term: &mut T) -> anyhow::Result<()>;
    fn start_fetching(&mut self, fetch_sem: Arc<Semaphore>) -> anyhow::Result<()>;
    fn render_content(&mut self, term: &mut T) -> anyhow::Result<()>;
    fn go_to_first_card(&mut self);
    fn go_to_next_card(&mut self) -> anyhow::Result<()>;
    fn reset(&mut self);

    fn has_titles(&self) -> bool {
        true
    }
}

pub struct Pager<T, H> {
    pages: Vec<Box<dyn Page<T, H>>>,
    page_index: usize,
    fetch_sem: Arc<Semaphore>,
}

#[allow(clippy::new_without_default)]
impl<T: Terminal, H: HttpClient> Pager<T, H> {
    pub fn new() -> Self {
        Self {
            pages: vec![],
            page_index: 0,
            fetch_sem: Arc::new(Semaphore::new(100)),
        }
    }

    pub fn add_page(&mut self, page: Box<dyn Page<T, H>>) {
        self.pages.push(page);
    }

    pub fn go_to_first_page(&mut self) -> anyhow::Result<()> {
        self.page_index = 0;
        self.current_page()?.reset();
        Ok(())
    }

    pub fn go_to_next_page(&mut self) -> anyhow::Result<()> {
        self.page_index += 1;
        if self.page_index >= self.pages.len() {
            self.go_to_first_page()?;
        } else {
            self.current_page()?.reset();
        }

        #[cfg(feature = "lazy_fetching")]
        {
            let fetch_sem = self.fetch_sem.clone();
            self.current_page()?.start_fetching(fetch_sem)?;
        }

        #[cfg(not(feature = "lazy_fetching"))]
        if self.page_index == 1 {
            for page in self.pages.iter_mut() {
                let fetch_sem = self.fetch_sem.clone();
                page.start_fetching(fetch_sem)?;
            }
        }

        Ok(())
    }

    pub fn go_to_next_card(&mut self) -> anyhow::Result<()> {
        let page = self.current_page()?;
        page.go_to_next_card()?;
        Ok(())
    }

    pub fn render_initial(&mut self, term: &mut T) -> anyhow::Result<bool> {
        let page = self.current_page()?;

        let has_title = page.has_titles();
        if has_title {
            page.render_title(term)?;
        } else {
            page.render_content(term)?;
        }
        Ok(has_title)
    }

    pub fn render_following(&mut self, term: &mut T) -> anyhow::Result<()> {
        let page = self.current_page()?;
        page.render_content(term)
    }

    fn current_page(&mut self) -> anyhow::Result<&mut Box<dyn Page<T, H>>> {
        self.pages
            .get_mut(self.page_index)
            .context("Invalid page index")
    }
}
