mod create_table;
mod drop_table;
mod filter;
mod insert;
mod nothing;
mod projection;
mod scan;

use create_table::CreateTable;
use drop_table::DropTable;
use filter::Filter;
use insert::Insert;
use nothing::Nothing;
use projection::Projection;
use scan::Scan;

use super::plan::Node;
use super::types::Row;
use super::Storage;
use crate::Error;

/// A plan executor
pub trait Executor: Sync + Send + 'static {
    //fn affected(&self) -> Option<u64>;
    fn close(&mut self);
    //fn columns(&self) -> Vec<String>;
    fn fetch(&mut self) -> Result<Option<Row>, Error>;
}

impl dyn Executor {
    /// Executes a plan node, consuming it
    pub fn execute(ctx: &mut Context, node: Node) -> Result<Box<Self>, Error> {
        Ok(match node {
            Node::CreateTable { schema } => CreateTable::execute(ctx, schema)?,
            Node::DropTable { name } => DropTable::execute(ctx, name)?,
            Node::Filter { source, predicate } => {
                let source = Self::execute(ctx, *source)?;
                Filter::execute(ctx, source, predicate)?
            }
            Node::Insert { table, columns, expressions } => {
                Insert::execute(ctx, &table, columns, expressions)?
            }
            Node::Nothing => Nothing::execute(ctx)?,
            Node::Projection { source, labels, expressions } => {
                let source = Self::execute(ctx, *source)?;
                Projection::execute(ctx, source, labels, expressions)?
            }
            Node::Scan { table } => Scan::execute(ctx, table)?,
        })
    }
}

impl Iterator for dyn Executor {
    type Item = Result<Row, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fetch().transpose()
    }
}

/// A plan execution context
pub struct Context {
    /// The underlying storage
    pub storage: Box<Storage>,
}
