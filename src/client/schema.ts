import {PublicKey, SOLANA_SCHEMA} from '@solana/web3.js';

import * as borsh from '@quantleaf/borsh';
import { field, generateSchemas, variant } from '@quantleaf/borsh/lib/schema';

/**
 * The state of a greeting account managed by the hello world program
 */
export class ChannelAccount {

    @field({type: 'String'})
    name:string = "New channel";

    @field({type: PublicKey})
    tail_message:PublicKey|undefined = undefined

    constructor(fields: {name: string, tail_message: PublicKey}) {
        if (fields) {
            this.name = fields.name;
            this.tail_message = fields.tail_message
        }
    }
}

// A message instruction could potentially be many things, like string, image, videos, and so on.
export class Message {}

@variant(0)
export class MessageString extends Message
{
    string:string = ""   
}

/**
 * The state of a greeting account managed by the hello world program
 */
 export class MessageAccount {

    @field({type: PublicKey})
    from:PublicKey|undefined = undefined

    @field({type: PublicKey})
    next:PublicKey|undefined = undefined

    @field({type: MessageString})
    message:Message|undefined = undefined;

    @field({type: 'u64'})
    size: number|undefined = undefined;

    @field({type: 'u64'})
    parts: number|undefined = undefined;
 

    constructor(fields: {from: PublicKey, next:PublicKey, message: Message, parts:number }) {
        if (fields) {
            this.from = fields.from;
            this.next = fields.next
            this.message = fields.message
            this.parts = fields.parts
            this.size = 0 // temp

        }
    }
}



/* export const addSolchatSchema = (schema: Schema) =>  {
    
    schema.set(PublicKey, {
        kind: 'struct',
        fields: [['_bn', 'u256']],
    });
    
    schema.set(ChannelAccount, {
        kind: 'struct',
        fields:  [['name', 'string'],['tail_message', { kind: 'option', type: PublicKey }]],
    });

    return schema
    
} 
export const SOLCHAT_SCHEMA = addSolchatSchema(SOLANA_SCHEMA) 

*/

/**
 * The expected size of each greeting account.
 */

const addDefaultSchemas = (schemas:Map<any,any>) => {
    schemas.set(PublicKey, {
        kind: 'struct',
        fields: [['_bn', 'u256']],
    });
    return schemas
}
export const SCHEMAS = addDefaultSchemas(generateSchemas([ChannelAccount]));



